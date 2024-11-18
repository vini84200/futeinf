use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Jogo::Table)
                    .col(
                        ColumnDef::new(Jogo::Id)
                            .integer()
                            .auto_increment()
                            .primary_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Jogo::Nome).string().not_null())
                    .col(ColumnDef::new(Jogo::Data).date().not_null())
                    .col(ColumnDef::new(Jogo::Local).string().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Jogo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Jogo {
    Table,
    Id,
    Nome,
    Data,
    Local,
}
