pub mod add;
pub mod rm;
pub mod get;
pub mod makeadmin;
pub mod settings;
pub mod help;

use std::sync::Arc;
use teloxide::types::Message;
use teloxide::requests::Requester;

use crate::service::AppService;

/// Общий State для всех хендлеров — шарится через Arc.
#[derive(Clone)]
pub struct State {
    pub svc: Arc<AppService>,
}

impl State {
    pub fn new(svc: AppService) -> Self {
        Self { svc: Arc::new(svc) }
    }
}

// ── Вспомогательные функции ───────────────────────────────────────────────────

/// Возвращает chat_id из сообщения.
pub fn chat_id(msg: &Message) -> i64 {
    msg.chat.id.0
}

/// Возвращает user_id отправителя.
pub fn sender_id(msg: &Message) -> Option<i64> {
    msg.from.as_ref().map(|u| u.id.0 as i64)
}

/// Возвращает "@username" или числовой ID как строку.
pub fn sender_display(msg: &Message) -> String {
    msg.from
        .as_ref()
        .map(|u| {
            if let Some(un) = &u.username {
                un.clone()
            } else {
                u.id.0.to_string()
            }
        })
        .unwrap_or_else(|| "unknown".into())
}

/// thread_id топика в котором написано сообщение.
pub fn thread_id(msg: &Message) -> Option<i32> {
    msg.thread_id.map(|t| t.0)
}

/// Парсит аргументы после команды: "/add 18090 krx" → ["18090", "krx"]
pub fn parse_args(text: &str) -> Vec<&str> {
    let mut parts = text.split_whitespace();
    parts.next(); // пропускаем команду
    parts.collect()
}

/// Отправляет ответ в тот же топик что и исходное сообщение.
pub async fn send(bot: &teloxide::Bot, msg: &Message, text: &str) -> Result<Message, teloxide::RequestError> {
    let mut req = bot.send_message(msg.chat.id, text);

    if let Some(tid) = msg.thread_id {
        req = req.message_thread_id(tid);
    }

    req.await
}