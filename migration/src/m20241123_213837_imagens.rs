use sea_orm_migration::{prelude::*, schema::*};

use crate::m20241020_003335_create_jogo_e_jogador::Jogador;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Jogador::Table)
                    .add_column(
                        ColumnDef::new(Jogador::Imagem)
                            .blob()
                    )
                    .to_owned()
                )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Jogador::Table)
                    .drop_column(Jogador::Imagem)
                    .to_owned()
            )
            .await
    }
}
