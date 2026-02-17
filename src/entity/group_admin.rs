use sea_orm::entity::prelude::*;

/// Пользователи которым разрешено использовать /add и /rm.
/// Добавляются через /makeadmin (только dev-admins).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "group_admins")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub chat_id: i64,
    pub user_id: i64,
    #[sea_orm(column_type = "String(StringLen::N(128))", nullable)]
    pub username: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}