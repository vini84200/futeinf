use sea_orm_migration::{prelude::*, schema::*};
use crate::m20241020_003335_create_jogo_e_jogador::Jogador;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(ListaExtra::Table)
                    .if_not_exists()
                    .col(pk_auto(ListaExtra::Id))
                    .col(ColumnDef::new(ListaExtra::JogadorId).integer().not_null())
                    .col(date_time(ListaExtra::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_jogador_id")
                            .from(ListaExtra::Table, ListaExtra::JogadorId)
                            .to(Jogador::Table, Jogador::Id),
                    )
                    .to_owned(),
            ).await?;
        
        manager
            .alter_table(
                Table::alter()
                    .table(Jogador::Table)
                    .add_column(ColumnDef::new(Jogador::Admin).boolean().not_null().default(false))
                    .to_owned()
            ).await?;
            

        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(ListaExtra::Table).to_owned()).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Jogador::Table)
                    .drop_column(Jogador::Admin)
                    .to_owned()
            ).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ListaExtra {
    Table,
    Id,
    JogadorId,
    Data
}
