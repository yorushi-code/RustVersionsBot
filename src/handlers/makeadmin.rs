use std::sync::Arc;
use teloxide::{Bot, RequestError};
use teloxide::requests::Requester;
use teloxide::types::{ChatMemberKind, Message};

use crate::service::AppService;
use super::{chat_id, sender_id};

/// /makeadmin
///
/// БЕЗ реплая → проходит по всем Telegram-администраторам группы
/// и регистрирует каждого в таблице group_admins.
///
/// С реплаем → регистрирует только того пользователя,
/// на чьё сообщение был сделан реплай.
///
/// Вызвать может только dev (захардкоженный ID + DEV_ADMIN_IDS в .env).
pub async fn handle(
    bot: Bot,
    msg: Message,
    svc: Arc<AppService>,
) -> Result<(), RequestError> {
    let cid = chat_id(&msg);
    let uid = match sender_id(&msg) {
        Some(id) => id,
        None => return Ok(()),
    };

    // /makeadmin не ограничен топиком — иначе нельзя будет инициализировать
    // права в группе перед тем как топик настроен.

    // ── Проверка прав ──────────────────────────────────────────────────────
    if !svc.is_dev(uid) {
        crate::handlers::send(
            &bot,
            &msg,
            "Permission denied. Only dev admins can use /makeadmin.",
        )
        .await?;
        return Ok(());
    }

    // ── Режим РЕПЛАЙ: один конкретный человек ──────────────────────────────
    if let Some(replied) = msg.reply_to_message() {
        return handle_reply(&bot, &msg, &svc, cid, replied).await;
    }

    // ── Режим БЕЗ РЕПЛАЯ: все Telegram-админы группы ──────────────────────
    handle_all(&bot, &msg, &svc, cid).await
}

// ── Реплай → один человек ─────────────────────────────────────────────────────

async fn handle_reply(
    bot: &Bot,
    msg: &Message,
    svc: &AppService,
    cid: i64,
    replied: &Message,
) -> Result<(), RequestError> {
    // from может быть None для анонимных сообщений каналов
    let Some(target_user) = &replied.from else {
        crate::handlers::send(bot, msg, "Cannot identify the user (anonymous message?).").await?;
        return Ok(());
    };

    // teloxide: UserId — это newtype над u64
    let target_id: i64 = target_user.id.0 as i64;
    let username = target_user.username.clone();

    let display = username
        .as_deref()
        .map(|u| format!("@{u}"))
        .unwrap_or_else(|| target_id.to_string());

    match svc.admins.upsert(cid, target_id, username).await {
        Ok(()) => {
            crate::handlers::send(
                bot,
                msg,
                &format!("Done. {display} can now use /add and /rm."),
            )
            .await?;
        }
        Err(e) => {
            tracing::error!("makeadmin reply: upsert({target_id}) failed: {e}");
            crate::handlers::send(bot, msg, "Internal error.").await?;
        }
    }

    Ok(())
}

// ── Без реплая → все Telegram-админы ─────────────────────────────────────────

async fn handle_all(
    bot: &Bot,
    msg: &Message,
    svc: &AppService,
    cid: i64,
) -> Result<(), RequestError> {
    // Получаем список администраторов от Telegram API
    let tg_admins = match bot.get_chat_administrators(msg.chat.id).await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!("get_chat_administrators for {cid}: {e}");
            crate::handlers::send(
                bot,
                msg,
                "Failed to fetch group admins from Telegram API.",
            )
            .await?;
            return Ok(());
        }
    };

    let mut registered: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();

    for member in &tg_admins {
        // Пропускаем ботов
        if member.user.is_bot {
            continue;
        }

        // Проверяем что статус Creator или Administrator
        // В teloxide 0.13 статус хранится в ChatMemberKind
        let is_admin = matches!(
            member.kind,
            ChatMemberKind::Owner(_) | ChatMemberKind::Administrator(_)
        );

        if !is_admin {
            continue;
        }

        let target_id: i64 = member.user.id.0 as i64;
        let username = member.user.username.clone();

        let display = username
            .as_deref()
            .map(|u| format!("@{u}"))
            .unwrap_or_else(|| target_id.to_string());

        match svc.admins.upsert(cid, target_id, username).await {
            Ok(()) => registered.push(display),
            Err(e) => {
                tracing::error!("makeadmin all: upsert({target_id}) failed: {e}");
                failed.push(display);
            }
        }
    }

    // Формируем ответ
    let mut lines: Vec<String> = Vec::new();

    if registered.is_empty() && failed.is_empty() {
        lines.push("No non-bot admins found in this group.".into());
    } else {
        if !registered.is_empty() {
            lines.push(format!(
                "Registered {} admin(s) with /add + /rm access:",
                registered.len()
            ));
            for name in &registered {
                lines.push(format!("  {name}"));
            }
        }

        if !failed.is_empty() {
            lines.push(format!("\nFailed to register ({} error(s)):", failed.len()));
            for name in &failed {
                lines.push(format!("  {name}"));
            }
        }
    }

    crate::handlers::send(bot, msg, &lines.join("\n")).await?;

    Ok(())
}