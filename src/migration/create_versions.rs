use sea_orm_migration::prelude::*;

// Каждый файл = одна таблица. Порядок в векторе ниже определяет очерёдность.
mod create_versions;
mod create_clients; // depends on versions (FK)
mod create_group_admins;
mod create_bot_settings;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(create_versions::Migration),
            Box::new(create_clients::Migration), // после versions
            Box::new(create_group_admins::Migration),
            Box::new(create_bot_settings::Migration),
        ]
    }
}