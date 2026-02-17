use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "create_group_admins"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupAdmins::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupAdmins::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(GroupAdmins::ChatId)
                            .big_integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(GroupAdmins::UserId)
                            .big_integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(GroupAdmins::Username)
                            .string_len(128)
                            .null()
                    )
                    .col(
                        ColumnDef::new(GroupAdmins::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .index(
                        Index::create()
                            .unique()
                            .col(GroupAdmins::ChatId)
                            .col(GroupAdmins::UserId)
                    )
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupAdmins::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum GroupAdmins {
    Table,
    Id,
    ChatId,
    UserId,
    Username,
    CreatedAt,
}