use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait,
    DatabaseConnection, EntityTrait, QueryFilter,
};
use serde_json;
use crate::entity::bot_settings;
use super::RepoResult;

pub struct SettingsRepo {
    db: DatabaseConnection,
}

impl SettingsRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Получить настройки чата, или None если ещё не созданы.
    pub async fn get(&self, chat_id: i64) -> RepoResult<Option<bot_settings::Model>> {
        Ok(bot_settings::Entity::find()
            .filter(bot_settings::Column::ChatId.eq(chat_id))
            .one(&self.db)
            .await?)
    }

    /// Получить или создать настройки с дефолтными значениями.
    pub async fn get_or_default(&self, chat_id: i64) -> RepoResult<bot_settings::Model> {
        if let Some(s) = self.get(chat_id).await? {
            return Ok(s);
        }

        let model = bot_settings::ActiveModel {
            chat_id: Set(chat_id),
            allowed_topic_id: Set(None),
            extra_admin_ids: Set(serde_json::json!([])),
            updated_at: Set(Utc::now()),
            updated_by: Set(0),
            ..Default::default()
        };

        Ok(model.insert(&self.db).await?)
    }

    /// Установить топик в котором работают команды (None = везде).
    pub async fn set_topic(
        &self,
        chat_id: i64,
        topic_id: Option<i32>,
        updated_by: i64,
    ) -> RepoResult<()> {
        let settings = self.get_or_default(chat_id).await?;

        let mut active: bot_settings::ActiveModel = settings.into();
        active.allowed_topic_id = Set(topic_id);
        active.updated_at = Set(Utc::now());
        active.updated_by = Set(updated_by);

        active.update(&self.db).await?;

        Ok(())
    }

    /// Добавить extra admin (идемпотентно).
    pub async fn add_extra_admin(
        &self,
        chat_id: i64,
        user_id: i64,
        updated_by: i64,
    ) -> RepoResult<()> {
        let settings = self.get_or_default(chat_id).await?;

        let mut ids: Vec<i64> = serde_json::from_value(settings.extra_admin_ids.clone())
            .unwrap_or_default();

        if !ids.contains(&user_id) {
            ids.push(user_id);
        }

        let mut active: bot_settings::ActiveModel = settings.into();
        active.extra_admin_ids = Set(serde_json::json!(ids));
        active.updated_at = Set(Utc::now());
        active.updated_by = Set(updated_by);

        active.update(&self.db).await?;

        Ok(())
    }

    /// Удалить extra admin.
    pub async fn remove_extra_admin(
        &self,
        chat_id: i64,
        user_id: i64,
        updated_by: i64,
    ) -> RepoResult<()> {
        let settings = self.get_or_default(chat_id).await?;

        let mut ids: Vec<i64> = serde_json::from_value(settings.extra_admin_ids.clone())
            .unwrap_or_default();

        ids.retain(|&id| id != user_id);

        let mut active: bot_settings::ActiveModel = settings.into();
        active.extra_admin_ids = Set(serde_json::json!(ids));
        active.updated_at = Set(Utc::now());
        active.updated_by = Set(updated_by);

        active.update(&self.db).await?;

        Ok(())
    }

    /// Получить список разрешённых топиков для чата (None = везде).
    pub async fn allowed_topic(&self, chat_id: i64) -> RepoResult<Option<i32>> {
        Ok(self.get(chat_id).await?.and_then(|s| s.allowed_topic_id))
    }

    /// Получить extra admin ids для чата.
    pub async fn extra_admins(&self, chat_id: i64) -> RepoResult<Vec<i64>> {
        let settings = match self.get(chat_id).await? {
            Some(s) => s,
            None => return Ok(vec![]),
        };

        Ok(serde_json::from_value(settings.extra_admin_ids).unwrap_or_default())
    }
}