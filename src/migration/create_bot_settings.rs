use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "create_bot_settings"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BotSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BotSettings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(BotSettings::ChatId)
                            .big_integer()
                            .not_null()
                            .unique_key()
                    )
                    // NULL = команды работают везде; число = только в этом топике
                    .col(
                        ColumnDef::new(BotSettings::AllowedTopicId)
                            .integer()
                            .null()
                    )
                    // JSON массив i64, например: [123456, 789012]
                    .col(
                        ColumnDef::new(BotSettings::ExtraAdminIds)
                            .json()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(BotSettings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(BotSettings::UpdatedBy)
                            .big_integer()
                            .not_null()
                    )
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BotSettings::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum BotSettings {
    Table,
    Id,
    ChatId,
    AllowedTopicId,
    ExtraAdminIds,
    UpdatedAt,
    UpdatedBy,
}