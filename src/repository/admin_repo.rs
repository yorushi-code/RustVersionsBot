use chrono::Utc;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    sea_query::OnConflict,
    ActiveValue::Set,
};
use crate::entity::group_admin;
use super::RepoResult;

pub struct AdminRepo {
    db: DatabaseConnection,
}

impl AdminRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Добавить пользователя как group-admin (идемпотентно).
    /// INSERT ... ON CONFLICT (chat_id, user_id) DO NOTHING — атомарно, без гонок.
    pub async fn upsert(
        &self,
        chat_id: i64,
        user_id: i64,
        username: Option<String>,
    ) -> RepoResult<()> {
        let model = group_admin::ActiveModel {
            chat_id: Set(chat_id),
            user_id: Set(user_id),
            username: Set(username),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        // try_insert корректно обрабатывает конфликт — возвращает Ok(Conflicted)
        group_admin::Entity::insert(model)
            .on_conflict(
                OnConflict::columns([
                    group_admin::Column::ChatId,
                    group_admin::Column::UserId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .do_nothing() // ← говорит SeaORM не считать 0 затронутых строк ошибкой
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Проверить является ли пользователь group-admin для данного чата.
    pub async fn is_admin(&self, chat_id: i64, user_id: i64) -> RepoResult<bool> {
        let found = group_admin::Entity::find()
            .filter(group_admin::Column::ChatId.eq(chat_id))
            .filter(group_admin::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;

        Ok(found.is_some())
    }
}