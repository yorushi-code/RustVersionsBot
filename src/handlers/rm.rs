use teloxide::{Bot, RequestError};
use teloxide::requests::Requester;
use teloxide::types::Message;

use crate::repository::RepoError;
use crate::service::AppService;
use super::{chat_id, parse_args, sender_id, thread_id};

/// /rm <version> <client_name>
pub async fn handle(bot: Bot, msg: Message, svc: std::sync::Arc<AppService>) -> Result<(), RequestError> {
    let cid = chat_id(&msg);
    let uid = match sender_id(&msg) {
        Some(id) => id,
        None => return Ok(()),
    };

    if !svc.is_allowed_topic(cid, thread_id(&msg)).await {
        return Ok(());
    }

    if !svc.can_write(cid, uid).await {
        crate::handlers::send(&bot, &msg, "Permission denied.").await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args = parse_args(text);

    if args.len() < 2 {
        crate::handlers::send(
            &bot,
            &msg,
            "Usage: /rm <version> <client_name>\nExample: /rm 18090 krx",
        )
        .await?;
        return Ok(());
    }

    let version_num = match args[0].parse::<i32>() {
        Ok(n) => n,
        Err(_) => {
            crate::handlers::send(&bot, &msg, "Version must be a number.").await?;
            return Ok(());
        }
    };

    let name = args[1..].join(" ");

    // Ищем версию
    let ver = match svc.versions.find_with_clients(version_num).await {
        Ok(Some((v, _))) => v,
        Ok(None) => {
            crate::handlers::send(&bot, &msg, &format!("Version {version_num} — Not Found")).await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("rm: find version failed: {e}");
            crate::handlers::send(&bot, &msg, "Internal error.").await?;
            return Ok(());
        }
    };

    // Удаляем клиента
    match svc.clients.remove(ver.id, &name).await {
        Ok(()) => {
            // Если версия опустела — удаляем её тоже
            let _ = svc.versions.delete_if_empty(ver.id).await;
            crate::handlers::send(
                &bot,
                &msg,
                &format!("Removed: {:?} from version {}", name, version_num),
            )
            .await?;
        }
        Err(RepoError::NotFound) => {
            crate::handlers::send(
                &bot,
                &msg,
                &format!("Not found: {:?} in version {}", name, version_num),
            )
            .await?;
        }
        Err(e) => {
            tracing::error!("rm: remove client failed: {e}");
            crate::handlers::send(&bot, &msg, "Internal error.").await?;
        }
    }

    Ok(())
}