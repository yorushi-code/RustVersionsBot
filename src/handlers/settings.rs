use teloxide::{Bot, RequestError};
use teloxide::requests::Requester;
use teloxide::types::Message;

use crate::service::AppService;
use super::{chat_id, parse_args, sender_id, thread_id};

/// /settings — управление настройками бота для этого чата.
/// Доступно только dev-admins.
///
/// Субкоманды:
/// /settings → показать текущие настройки
/// /settings topic <id> → установить разрешённый топик
/// /settings topic off → отключить ограничение по топику
/// /settings addadmin <user_id> → добавить extra admin
/// /settings rmadmin <user_id> → убрать extra admin
pub async fn handle(bot: Bot, msg: Message, svc: std::sync::Arc<AppService>) -> Result<(), RequestError> {
    let cid = chat_id(&msg);
    let uid = match sender_id(&msg) {
        Some(id) => id,
        None => return Ok(()),
    };

    // Не ограничиваем settings по топику — иначе нельзя будет настроить топик первый раз
    // Но проверяем права dev
    if !svc.is_dev(uid) {
        crate::handlers::send(&bot, &msg, "Permission denied. Only dev admins can change settings.")
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args = parse_args(text);

    match args.as_slice() {
        // /settings — показать
        [] => {
            show_settings(&bot, &msg, &svc, cid).await?;
        }

        // /settings topic <id|off>
        ["topic", value] => {
            if value.eq_ignore_ascii_case("off") {
                svc.settings
                    .set_topic(cid, None, uid)
                    .await
                    .unwrap_or_else(|e| tracing::error!("set_topic: {e}"));

                crate::handlers::send(
                    &bot,
                    &msg,
                    "Topic restriction removed. Commands work in any topic now.",
                )
                .await?;
            } else {
                match value.parse::<i32>() {
                    Ok(tid) => {
                        svc.settings
                            .set_topic(cid, Some(tid), uid)
                            .await
                            .unwrap_or_else(|e| tracing::error!("set_topic: {e}"));

                        crate::handlers::send(
                            &bot,
                            &msg,
                            &format!("Commands restricted to topic ID {tid}."),
                        )
                        .await?;
                    }
                    Err(_) => {
                        crate::handlers::send(
                            &bot,
                            &msg,
                            "Topic ID must be a number or \"off\".",
                        )
                        .await?;
                    }
                }
            }
        }

        // /settings addadmin <user_id>
        ["addadmin", id_str] => {
            match id_str.parse::<i64>() {
                Ok(target_id) => {
                    svc.settings
                        .add_extra_admin(cid, target_id, uid)
                        .await
                        .unwrap_or_else(|e| tracing::error!("add_extra_admin: {e}"));

                    crate::handlers::send(
                        &bot,
                        &msg,
                        &format!("User {target_id} added as extra admin."),
                    )
                    .await?;
                }
                Err(_) => {
                    crate::handlers::send(
                        &bot,
                        &msg,
                        "Usage: /settings addadmin <user_id>",
                    )
                    .await?;
                }
            }
        }

        // /settings rmadmin <user_id>
        ["rmadmin", id_str] => {
            match id_str.parse::<i64>() {
                Ok(target_id) => {
                    svc.settings
                        .remove_extra_admin(cid, target_id, uid)
                        .await
                        .unwrap_or_else(|e| tracing::error!("remove_extra_admin: {e}"));

                    crate::handlers::send(
                        &bot,
                        &msg,
                        &format!("User {target_id} removed from extra admins."),
                    )
                    .await?;
                }
                Err(_) => {
                    crate::handlers::send(
                        &bot,
                        &msg,
                        "Usage: /settings rmadmin <user_id>",
                    )
                    .await?;
                }
            }
        }

        _ => {
            crate::handlers::send(&bot, &msg, settings_help()).await?;
        }
    }

    Ok(())
}

async fn show_settings(
    bot: &Bot,
    msg: &Message,
    svc: &AppService,
    cid: i64,
) -> Result<(), RequestError> {
    let settings = svc.settings.get_or_default(cid).await;

    let topic_line = match &settings {
        Ok(s) => match s.allowed_topic_id {
            Some(t) => format!("Allowed topic ID : {t}"),
            None => "Allowed topic : any (no restriction)".into(),
        },
        Err(_) => "Allowed topic : (error reading)".into(),
    };

    let extra_line = match svc.settings.extra_admins(cid).await {
        Ok(ids) if ids.is_empty() => "Extra admins : none".into(),
        Ok(ids) => format!(
            "Extra admins : {}",
            ids.iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Err(_) => "Extra admins : (error reading)".into(),
    };

    let text = format!(
        "Settings for this chat:\n\n{topic_line}\n{extra_line}\n\n{}",
        settings_help()
    );

    crate::handlers::send(bot, msg, &text).await?;

    Ok(())
}

fn settings_help() -> &'static str {
    "Commands:\n\
    /settings — show current settings\n\
    /settings topic <id> — restrict commands to topic ID\n\
    /settings topic off — allow commands in any topic\n\
    /settings addadmin <user_id> — grant /add+/rm to a user\n\
    /settings rmadmin <user_id> — revoke access"
}