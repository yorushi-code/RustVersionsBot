use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use crate::entity::{client, version};
use super::{RepoError, RepoResult};

pub struct VersionRepo {
    db: DatabaseConnection,
}

impl VersionRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Возвращает существующую версию или создаёт новую.
    pub async fn get_or_create(&self, number: i32) -> RepoResult<version::Model> {
        // Пробуем найти
        if let Some(v) = version::Entity::find()
            .filter(version::Column::Number.eq(number))
            .one(&self.db)
            .await?
        {
            return Ok(v);
        }

        // Создаём новую
        let model = version::ActiveModel {
            number: Set(number),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let inserted = model.insert(&self.db).await?;
        Ok(inserted)
    }

    /// Найти версию с загруженными клиентами.
    pub async fn find_with_clients(
        &self,
        number: i32,
    ) -> RepoResult<Option<(version::Model, Vec<client::Model>)>> {
        let Some(ver) = version::Entity::find()
            .filter(version::Column::Number.eq(number))
            .one(&self.db)
            .await?
        else {
            return Ok(None);
        };

        let clients = client::Entity::find()
            .filter(client::Column::VersionId.eq(ver.id))
            .order_by_asc(client::Column::Name)
            .all(&self.db)
            .await?;

        Ok(Some((ver, clients)))
    }

    /// Пагинированный список версий (newest first) с клиентами.
    pub async fn paginate(
        &self,
        page: u64,
        page_size: u64,
    ) -> RepoResult<(Vec<(version::Model, Vec<client::Model>)>, u64)> {
        let total = version::Entity::find().count(&self.db).await?;

        let offset = (page.saturating_sub(1)) * page_size;
        let versions = version::Entity::find()
            .order_by_desc(version::Column::Number)
            .offset(offset)
            .limit(page_size)
            .all(&self.db)
            .await?;

        let mut result = Vec::with_capacity(versions.len());

        for ver in versions {
            let clients = client::Entity::find()
                .filter(client::Column::VersionId.eq(ver.id))
                .order_by_asc(client::Column::Name)
                .all(&self.db)
                .await?;

            result.push((ver, clients));
        }

        Ok((result, total))
    }

    /// Удалить версию если у неё не осталось клиентов.
    pub async fn delete_if_empty(&self, version_id: i32) -> RepoResult<()> {
        let count = client::Entity::find()
            .filter(client::Column::VersionId.eq(version_id))
            .count(&self.db)
            .await?;

        if count == 0 {
            version::Entity::delete_by_id(version_id)
                .exec(&self.db)
                .await?;
        }

        Ok(())
    }

    /// Найти версию по первичному ключу с клиентами.
    pub async fn find_with_clients_by_id(
        &self,
        id: i32,
    ) -> RepoResult<Option<(version::Model, Vec<client::Model>)>> {
        let Some(ver) = version::Entity::find_by_id(id).one(&self.db).await? else {
            return Ok(None);
        };

        let clients = client::Entity::find()
            .filter(client::Column::VersionId.eq(ver.id))
            .order_by_asc(client::Column::Name)
            .all(&self.db)
            .await?;

        Ok(Some((ver, clients)))
    }
}