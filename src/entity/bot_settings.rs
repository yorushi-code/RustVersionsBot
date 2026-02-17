use sea_orm::entity::prelude::*;

/// Настройки бота для конкретного чата.
/// Одна запись на chat_id.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bot_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub chat_id: i64,
    /// Telegram message_thread_id топика в котором работают команды.
    /// None = команды работают в любом топике/чате.
    pub allowed_topic_id: Option<i32>,
    /// Дополнительные user_id которым разрешены /add и /rm (через /settings).
    /// Хранится как JSON: [123456, 789012]
    pub extra_admin_ids: Json,
    pub updated_at: DateTimeUtc,
    pub updated_by: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}