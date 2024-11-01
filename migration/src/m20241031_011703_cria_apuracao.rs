use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Apuracao::Table)
                    .col(
                        ColumnDef::new(Apuracao::Id)
                            .integer()
                            .auto_increment()
                            .primary_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Apuracao::WeekId)
                            .integer()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Apuracao::RandomId)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Apuracao::State).string().not_null())
                    .col(ColumnDef::new(Apuracao::Results).json().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Apuracao::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Apuracao {
    Table,
    Id,
    WeekId,
    RandomId,
    State,
    Results,
}
