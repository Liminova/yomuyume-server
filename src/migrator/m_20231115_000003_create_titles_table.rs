use axum::async_trait;
use sea_orm_migration::prelude::*;

use super::m_20231115_000002_create_categories_table::Categories;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20231115_000003_create_titles_table"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Titles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Titles::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Titles::Title).string().not_null())
                    .col(ColumnDef::new(Titles::CategoriesId).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-title-category_id")
                            .from(Titles::Table, Titles::CategoriesId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Titles::Author).string())
                    .col(ColumnDef::new(Titles::Description).string())
                    .col(ColumnDef::new(Titles::ReleaseDate).date_time())
                    .col(ColumnDef::new(Titles::IsColored).boolean())
                    .col(ColumnDef::new(Titles::IsCompleted).boolean())
                    .col(ColumnDef::new(Titles::Thumbnail).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Titles::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Titles {
    Table,
    Id,
    Title,
    CategoriesId,
    Author,
    Description,
    ReleaseDate,
    IsColored,
    IsCompleted,
    Thumbnail,
}
