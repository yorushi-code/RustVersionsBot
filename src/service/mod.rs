use std::sync::Arc;

use crate::{
    config::Config,
    entity::{client, version},
    repository::{AdminRepo, ClientRepo, RepoError, SettingsRepo, VersionRepo},
};

/// Центральный сервисный слой. Хранится в Arc и шарится между хендлерами.
pub struct AppService {
    pub versions: Arc<VersionRepo>,
    pub clients: Arc<ClientRepo>,
    pub admins: Arc<AdminRepo>,
    pub settings: Arc<SettingsRepo>,
    pub config: Arc<Config>,
}

impl AppService {
    pub fn new(
        versions: VersionRepo,
        clients: ClientRepo,
        admins: AdminRepo,
        settings: SettingsRepo,
        config: Arc<Config>,
    ) -> Self {
        Self {
            versions: Arc::new(versions),
            clients: Arc::new(clients),
            admins: Arc::new(admins),
            settings: Arc::new(settings),
            config,
        }
    }

    // ── Permission checks ─────────────────────────────────────────────────────

    /// Только dev-admins (захардкоженные IDs).
    pub fn is_dev(&self, user_id: i64) -> bool {
        self.config.is_dev(user_id)
    }

    /// Может использовать /add и /rm:
    /// dev-admin OR group-admin (через /makeadmin) OR extra_admin (через /settings)
    pub async fn can_write(&self, chat_id: i64, user_id: i64) -> bool {
        if self.is_dev(user_id) {
            return true;
        }

        // Проверяем group_admins
        if let Ok(true) = self.admins.is_admin(chat_id, user_id).await {
            return true;
        }

        // Проверяем extra_admins из настроек
        if let Ok(ids) = self.settings.extra_admins(chat_id).await {
            if ids.contains(&user_id) {
                return true;
            }
        }

        false
    }

    /// Проверяет что сообщение пришло из разрешённого топика.
    /// None = разрешён любой топик.
    pub async fn is_allowed_topic(&self, chat_id: i64, thread_id: Option<i32>) -> bool {
        match self.settings.allowed_topic(chat_id).await {
            Ok(None) => true, // ограничений нет
            Ok(Some(allowed)) => thread_id == Some(allowed),
            Err(_) => true, // при ошибке БД не блокируем
        }
    }

    // ── Formatting ────────────────────────────────────────────────────────────

    /// Форматирует один блок версии:
    /// ```
    /// ❯ Version: 18090
    /// • krx[bot][the_yorushi]
    /// • ddnet[legit][the_yorushi]
    /// ```
    pub fn format_version_block(ver: &version::Model, clients: &[client::Model]) -> String {
        let mut s = format!("❯ Version: {}\n", ver.number);

        for c in clients {
            s.push_str(&format!(" • {}\n", c.display()));
        }

        s
    }

    /// Форматирует пагинированный список версий.
    pub fn format_page(
        versions: &[(version::Model, Vec<client::Model>)],
        total: u64,
        page: u64,
        page_size: u64,
    ) -> String {
        let mut s = format!("Total versions: {}\n\n", total);

        for (ver, clients) in versions {
            s.push_str(&Self::format_version_block(ver, clients));
            s.push('\n');
        }

        let start = (page - 1) * page_size + 1;
        let end = start + versions.len() as u64 - 1;
        let total_pages = total.div_ceil(page_size);

        s.push_str(&format!("\nShowing {}-{} of {}", start, end, total));

        if page < total_pages {
            s.push_str(&format!("\nUse /get page {} to see more", page + 1));
        }

        s
    }

    /// Парсит "krx[bot]" → ("krx", Some("bot"))
    /// или "ddnet" → ("ddnet", None)
    pub fn parse_name_type(raw: &str) -> (String, Option<String>) {
        let raw = raw.trim();

        if let Some(bracket) = raw.find('[') {
            if let Some(close) = raw.find(']') {
                if close > bracket {
                    let name = raw[..bracket].trim().to_string();
                    let ctype = raw[bracket + 1..close].trim().to_lowercase();
                    let ctype = if ctype.is_empty() { None } else { Some(ctype) };

                    return (name, ctype);
                }
            }
        }

        (raw.to_string(), None)
    }
}