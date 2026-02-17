use teloxide::{Bot, RequestError};
use teloxide::requests::Requester;
use teloxide::types::Message;

use crate::service::AppService;
use super::{sender_id, thread_id, chat_id};

pub async fn handle(bot: Bot, msg: Message, svc: std::sync::Arc<AppService>) -> Result<(), RequestError> {
    let uid = sender_id(&msg).unwrap_or(0);
    let cid = chat_id(&msg);

    let is_writer = svc.can_write(cid, uid).await;
    let is_dev = svc.is_dev(uid);

    let mut lines = vec![
        "Available commands:",
        "",
        "/start — welcome",
        "/help — this message",
        "/get — list all versions (paginated)",
        "/get page <n> — specific page",
        "/get <version> — show one version",
        "/get <client> — search by client name",
        "/get <ver> <client> — check client in version",
    ];

    if is_writer {
        lines.extend_from_slice(&[
            "",
            "-- Write access --",
            "/add <ver> <client> — add client (e.g. /add 18090 krx[bot])",
            "/rm <ver> <client> — remove client",
        ]);
    }

    if is_dev {
        lines.extend_from_slice(&[
            "",
            "-- Dev admin --",
            "/makeadmin — make all group admins bot-admins",
            "/makeadmin (reply) — make the replied-to user a bot-admin",
            "/settings — show/change bot settings",
        ]);
    }

    let text = lines.join("\n");

    let mut req = bot
        .send_message(msg.chat.id, format!("<pre>{}</pre>", text))
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_to_message_id(msg.id);

    if let Some(tid) = msg.thread_id {
        req = req.message_thread_id(tid);
        req.await?;
    } else {
        req.await?;
    }

    Ok(())
}