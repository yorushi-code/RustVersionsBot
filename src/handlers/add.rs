use std::sync::Arc;
use teloxide::{Bot, RequestError};
use teloxide::requests::Requester;
use teloxide::types::{Message, ParseMode};

use crate::repository::RepoError;
use crate::service::AppService;
use super::{chat_id, parse_args, sender_display, sender_id, thread_id};

/// /add <version> <client_name[type]>
///
/// Примеры:
/// /add 18090 krx[bot]
/// /add 18090 ddnet
pub async fn handle(bot: Bot, msg: Message, svc: Arc<AppService>) -> Result<(), RequestError> {
    let cid = chat_id(&msg);
    let uid = match sender_id(&msg) {
        Some(id) => id,
        None => return Ok(()),
    };

    // Тихо игнорируем если не тот топик
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
            "Usage: /add <version> <client_name>\nExample: /add 18090 krx[bot]",
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

    let raw_name = args[1..].join(" ");
    let (name, ctype) = AppService::parse_name_type(&raw_name);
    let author = sender_display(&msg);

    let ver = match svc.versions.get_or_create(version_num).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("add: get_or_create: {e}");
            crate::handlers::send(&bot, &msg, "Internal error.").await?;
            return Ok(());
        }
    };

    match svc.clients.add(ver.id, &name, ctype, Some(author), uid).await {
        Ok(client) => {
            crate::handlers::send(
                &bot,
                &msg,
                &format!("Added: {} to version {}.", client.display(), ver.number),
            )
            .await?;
        }
        Err(RepoError::AlreadyExists) => {
            crate::handlers::send(
                &bot,
                &msg,
                &format!("Already exists: {:?} in version {}.", name, ver.number),
            )
            .await?;
        }
        Err(e) => {
            tracing::error!("add: insert client: {e}");
            crate::handlers::send(&bot, &msg, "Internal error.").await?;
        }
    }

    Ok(())
}