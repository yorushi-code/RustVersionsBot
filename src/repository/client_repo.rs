use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, sea_query::OnConflict,
};
use crate::entity::client;
use super::{RepoError, RepoResult};

pub struct ClientRepo {
    db: DatabaseConnection,
}

impl ClientRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Добавить клиента. Возвращает RepoError::AlreadyExists если (version_id, name) уже есть
    pub async fn add(
        &self,
        version_id: i32,
        name: &str,
        client_type: Option<String>,
        author: Option<String>,
        added_by: i64,
    ) -> RepoResult<client::Model> {
        let model = client::ActiveModel {
            version_id: Set(version_id),
            name: Set(name.to_string()),
            client_type: Set(client_type),
            author: Set(author),
            added_by: Set(added_by),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        match model.insert(&self.db).await {
            Ok(inserted) => Ok(inserted),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("duplicate") || msg.contains("unique") || msg.contains("23505") {
                    Err(RepoError::AlreadyExists)
                } else {
                    Err(RepoError::Db(e))
                }
            }
        }
    }

    /// Удалить клиента по имени (case-insensitive) в рамках версии.
    pub async fn remove(&self, version_id: i32, name: &str) -> RepoResult<()> {
        let name_lower = name.to_lowercase();

        // Ищем с учётом регистра через LOWER()
        let found = client::Entity::find()
            .filter(client::Column::VersionId.eq(version_id))
            .all(&self.db)
            .await?;

        let target = found
            .into_iter()
            .find(|c| c.name.to_lowercase() == name_lower)
            .ok_or(RepoError::NotFound)?;

        client::Entity::delete_by_id(target.id)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Найти все клиенты с данным именем (case-insensitive) во всех версиях,
    /// отсортированные по убыванию version_id.
    pub async fn find_by_name(&self, name: &str) -> RepoResult<Vec<client::Model>> {
        let name_lower = name.to_lowercase();

        let all = client::Entity::find()
            .order_by_desc(client::Column::VersionId)
            .all(&self.db)
            .await?;

        let filtered = all
            .into_iter()
            .filter(|c| c.name.to_lowercase() == name_lower)
            .collect();

        Ok(filtered)
    }
}