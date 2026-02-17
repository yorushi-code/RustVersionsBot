use sea_orm::entity::prelude::*;

/// Клиент привязанный к версии.
/// Формат отображения: name[type][author]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "clients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub version_id: i32,
    /// Имя клиента, напр. "krx", "ddnet"
    #[sea_orm(column_type = "String(StringLen::N(128))")]
    pub name: String,
    /// Тип: "legit", "bot" или пустая строка
    #[sea_orm(column_type = "String(StringLen::N(32))", nullable)]
    pub client_type: Option<String>,
    /// @username или числовой ID того кто добавил
    #[sea_orm(column_type = "String(StringLen::N(128))", nullable)]
    pub author: Option<String>,
    /// Telegram user_id того кто добавил (для аудита)
    pub added_by: i64,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::version::Entity",
        from = "Column::VersionId",
        to = "super::version::Column::Id",
        on_delete = "Cascade"
    )]
    Version,
}

impl Related<super::version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Version.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Форматирует клиента как "name[type][author]"
    pub fn display(&self) -> String {
        let mut s = self.name.clone();
        
        if let Some(t) = &self.client_type {
            if !t.is_empty() {
                s.push('[');
                s.push_str(t);
                s.push(']');
            }
        }
        
        if let Some(a) = &self.author {
            if !a.is_empty() {
                s.push('[');
                s.push_str(a);
                s.push(']');
            }
        }
        
        s
    }
}