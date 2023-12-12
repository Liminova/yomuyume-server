use axum::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231212_000008_create_thumbnails_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Thumbnails::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Thumbnails::Id)
                    .integer()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Thumbnails::Path).string().not_null())
            .col(ColumnDef::new(Thumbnails::Hash).string().not_null())
            .col(ColumnDef::new(Thumbnails::Width).integer().not_null())
            .col(ColumnDef::new(Thumbnails::Height).integer().not_null())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Thumbnails::Table).to_owned();
        manager.drop_table(table).await
    }
}

#[derive(Iden)]
pub enum Thumbnails {
    Table,
    Id,
    Path,
    Hash,
    Width,
    Height,
}