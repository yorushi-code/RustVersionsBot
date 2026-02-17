use teloxide::{Bot, RequestError};
use teloxide::requests::Requester;
use teloxide::types::Message;

use crate::repository::RepoError;
use crate::service::AppService;
use super::{chat_id, parse_args, thread_id};

const MAX_MSG_LEN: usize = 3800;

/// /get → страница 1
/// /get page <n> → страница n
/// /get <число> → конкретная версия
/// /get <строка> → поиск по имени клиента
/// /get <число> <строка> → клиент в конкретной версии
pub async fn handle(bot: Bot, msg: Message, svc: std::sync::Arc<AppService>) -> Result<(), RequestError> {
    let cid = chat_id(&msg);

    if !svc.is_allowed_topic(cid, thread_id(&msg)).await {
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args = parse_args(text);

    let response = if args.is_empty() {
        // /get → страница 1
        get_page(&svc, 1).await
    } else if args[0].eq_ignore_ascii_case("page") {
        // /get page <n>
        let n = args.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1);
        get_page(&svc, n).await
    } else if let Ok(num) = args[0].parse::<i32>() {
        if args.len() >= 2 {
            // /get <число> <клиент>
            get_version_client(&svc, num, &args[1..].join(" ")).await
        } else {
            // /get <число>
            get_version(&svc, num).await
        }
    } else {
        // /get <строка>
        get_client(&svc, &args.join(" ")).await
    };

    send_long(&bot, &msg, &response).await
}

// ── Вспомогательные функции ───────────────────────────────────────────────────

async fn get_page(svc: &AppService, page: u64) -> String {
    let page = page.max(1);

    match svc.versions.paginate(page, svc.config.page_size).await {
        Ok((versions, total)) => {
            if total == 0 {
                return "No versions recorded yet.".into();
            }

            let total_pages = total.div_ceil(svc.config.page_size);

            if page > total_pages {
                return format!("Page {page} does not exist. Total pages: {total_pages}.");
            }

            AppService::format_page(&versions, total, page, svc.config.page_size)
        }
        Err(e) => {
            tracing::error!("get_page: {e}");
            "Internal error.".into()
        }
    }
}

async fn get_version(svc: &AppService, number: i32) -> String {
    match svc.versions.find_with_clients(number).await {
        Ok(Some((ver, clients))) => {
            let block = AppService::format_version_block(&ver, &clients);
            block.trim_end().to_string()
        }
        Ok(None) => format!("Version {number} — Not Found"),
        Err(e) => {
            tracing::error!("get_version: {e}");
            "Internal error.".into()
        }
    }
}

async fn get_client(svc: &AppService, name: &str) -> String {
    match svc.clients.find_by_name(name).await {
        Ok(clients) if clients.is_empty() => {
            format!("{name:?} — Not Found")
        }
        Ok(clients) => {
            // Нам нужны version numbers — грузим их
            let mut lines = vec![format!("Client {name:?} found in {} version(s):\n", clients.len())];

            for c in &clients {
                // version_id у нас есть, но нет number — делаем доп запрос
                match svc.versions.find_with_clients_by_id(c.version_id).await {
                    Ok(Some((ver, _))) => {
                        lines.push(format!(" Version {} → {}", ver.number, c.display()));
                    }
                    _ => {
                        lines.push(format!(" Version #{} → {}", c.version_id, c.display()));
                    }
                }
            }

            lines.join("\n")
        }
        Err(e) => {
            tracing::error!("get_client: {e}");
            "Internal error.".into()
        }
    }
}

async fn get_version_client(svc: &AppService, number: i32, name: &str) -> String {
    match svc.versions.find_with_clients(number).await {
        Ok(None) => format!("Version {number} — Not Found"),
        Ok(Some((ver, clients))) => {
            let name_lower = name.to_lowercase();

            match clients.iter().find(|c| c.name.to_lowercase() == name_lower) {
                Some(c) => format!("Version {} → {} (present)", ver.number, c.display()),
                None => format!("Version {} → {name:?} Not Found", ver.number),
            }
        }
        Err(e) => {
            tracing::error!("get_version_client: {e}");
            "Internal error.".into()
        }
    }
}

/// Отправляет длинный текст разбив на части если нужно (Telegram лимит ~4096).
async fn send_long(bot: &Bot, msg: &Message, text: &str) -> Result<(), RequestError> {
    let chunks = split_message(text, MAX_MSG_LEN);

    for chunk in chunks {
        let mut req = bot
            .send_message(msg.chat.id, format!("<pre>{}</pre>", escape_html(&chunk)))
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_to_message_id(msg.id);

        if let Some(tid) = msg.thread_id {
            req = req.message_thread_id(tid);
        }

        req.await?;
    }

    Ok(())
}

fn split_message(text: &str, max_len: usize) -> Vec<String> {
    if text.len() <= max_len {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        // Пробуем сплитить на пустых строках (между версиями)
        if current.len() + line.len() + 1 > max_len && !current.is_empty() {
            chunks.push(current.trim_end().to_string());
            current = String::new();
        }

        current.push_str(line);
        current.push('\n');
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim_end().to_string());
    }

    chunks
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}