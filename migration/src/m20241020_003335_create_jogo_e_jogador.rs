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
                    .table(Jogador::Table)
                    .col(
                        ColumnDef::new(Jogador::Id)
                            .integer()
                            .auto_increment()
                            .primary_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Jogador::Nome)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Jogador::Apelido)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Jogador::Email)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Jogador::SenhaHash)
                            .string()
                            .not_null(),
                    )
                    .to_owned()
            ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Jogador::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub(crate) enum Jogador {
    Table,
    Id,
    Nome,
    Apelido,
    Email,
    SenhaHash,
    // Added by m20241117_150055_lista_extra.rs
    Admin,
}
