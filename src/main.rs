mod config;
mod entity;
mod migration;
mod repository;
mod service;
mod handlers;

use std::sync::Arc;
use anyhow::Result;
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::MigratorTrait;
use teloxide::prelude::*;
use teloxide::types::Update;
use tracing_subscriber::{EnvFilter, fmt};

use config::Config;
use repository::{AdminRepo, ClientRepo, SettingsRepo, VersionRepo};
use service::AppService;
use handlers::State;

#[tokio::main]
async fn main() -> Result<()> {
    // ── Logging ───────────────────────────────────────────────────────────────
    fmt()
        .with_env_filter(
            EnvFilter::try_from_env("LOG_LEVEL")
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    
    tracing::info!("Starting VersionsBot");

    // ── Config ────────────────────────────────────────────────────────────────
    let cfg = Arc::new(Config::from_env()?);
    tracing::info!("Dev admin IDs: {:?}", cfg.dev_admin_ids);

    // ── Database ──────────────────────────────────────────────────────────────
    let mut opts = ConnectOptions::new(cfg.database_url.clone());
    opts
        // Аналог SetMaxOpenConns в Go
        .max_connections(cfg.db_max_connections)
        // Аналог SetMaxIdleConns — держим горячие соединения
        .min_connections(2)
        // Аналог SetConnMaxLifetime — 5 минут, чтобы не поймать silent disconnect
        .max_lifetime(std::time::Duration::from_secs(300))
        // Аналог SetConnMaxIdleTime
        .idle_timeout(std::time::Duration::from_secs(120))
        // Таймаут на соединение
        .connect_timeout(std::time::Duration::from_secs(10))
        .sqlx_logging(false);

    let db = Database::connect(opts).await?;
    tracing::info!("Database connected");

    // ── Migrations ────────────────────────────────────────────────────────────
    migration::Migrator::up(&db, None).await?;
    tracing::info!("Migrations applied");

    // ── Repositories ─────────────────────────────────────────────────────────
    let version_repo = VersionRepo::new(db.clone());
    let client_repo = ClientRepo::new(db.clone());
    let admin_repo = AdminRepo::new(db.clone());
    let settings_repo = SettingsRepo::new(db.clone());

    // ── Service ───────────────────────────────────────────────────────────────
    let svc = AppService::new(
        version_repo,
        client_repo,
        admin_repo,
        settings_repo,
        cfg.clone(),
    );

    // ── Bot ───────────────────────────────────────────────────────────────────
    let bot = Bot::new(&cfg.telegram_token);
    let state = State::new(svc);
    tracing::info!("Bot started");

    // ── Dispatcher ────────────────────────────────────────────────────────────
    // Используем ручной dispatch через filter_command для максимальной гибкости.
    // Каждый хендлер получает Arc<AppService> — это дёшево (просто клон Arc).
    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(route_command),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

// ── Command enum ─────────────────────────────────────────────────────────────
#[derive(teloxide::utils::command::BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Welcome")]
    Start,
    
    #[command(description = "Help")]
    Help,
    
    #[command(description = "Add client to version")]
    Add,
    
    #[command(description = "Remove client from version")]
    Rm,
    
    #[command(description = "Get version info")]
    Get,
    
    #[command(description = "Register admins")]
    MakeAdmin,
    
    #[command(description = "Bot settings (dev only)")]
    Settings,
}

// ── Router ────────────────────────────────────────────────────────────────────
async fn route_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: State,
) -> Result<(), teloxide::RequestError> {
    let svc = Arc::clone(&state.svc);
    
    match cmd {
        Command::Start => {
            let req = bot
                .send_message(msg.chat.id, "VersionsBot — tracks game client versions.\n\nType /help for commands.")
                .reply_to_message_id(msg.id);
            
            if let Some(tid) = msg.thread_id {
                req.message_thread_id(tid).await?;
            } else {
                req.await?;
            }
        }
        
        Command::Help => handlers::help::handle(bot, msg, svc).await?,
        Command::Add => handlers::add::handle(bot, msg, svc).await?,
        Command::Rm => handlers::rm::handle(bot, msg, svc).await?,
        Command::Get => handlers::get::handle(bot, msg, svc).await?,
        Command::MakeAdmin => handlers::makeadmin::handle(bot, msg, svc).await?,
        Command::Settings => handlers::settings::handle(bot, msg, svc).await?,
    }
    
    Ok(())
}