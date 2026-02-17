use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub telegram_token: String,
    
    /// Telegram user IDs с полными правами (могут /makeadmin, /settings).
    /// Захардкожен 6820351323 + список из DEV_ADMIN_IDS в env.
    pub dev_admin_ids: Vec<i64>,
    
    pub database_url: String,
    pub db_max_connections: u32,
    
    /// Количество версий на одной странице /get.
    pub page_size: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Загружаем .env (игнорируем ошибку если файла нет — в проде используем реальный env)
        let _ = dotenvy::dotenv();
        
        let telegram_token = std::env::var("TELEGRAM_TOKEN")
            .context("TELEGRAM_TOKEN is required")?;
        
        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL is required")?;
        
        let db_max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".into())
            .parse::<u32>()
            .context("DB_MAX_CONNECTIONS must be a number")?;
        
        let page_size = std::env::var("PAGE_SIZE")
            .unwrap_or_else(|_| "10".into())
            .parse::<u64>()
            .context("PAGE_SIZE must be a number")?;
        
        // Хардкод разработчика + опциональный список из env
        let mut dev_admin_ids: Vec<i64> = vec![6820351323];
        
        if let Ok(raw) = std::env::var("DEV_ADMIN_IDS") {
            for part in raw.split(',') {
                let part = part.trim();
                if part.is_empty() { 
                    continue; 
                }
                
                match part.parse::<i64>() {
                    Ok(id) => dev_admin_ids.push(id),
                    Err(_) => tracing::warn!("DEV_ADMIN_IDS: invalid id {:?}, skipped", part)
                }
            }
        }
        
        // Убираем дубли
        dev_admin_ids.sort_unstable();
        dev_admin_ids.dedup();
        
        Ok(Config {
            telegram_token,
            dev_admin_ids,
            database_url,
            db_max_connections,
            page_size,
        })
    }
    
    pub fn is_dev(&self, user_id: i64) -> bool {
        self.dev_admin_ids.contains(&user_id)
    }
}