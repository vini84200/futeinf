pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20241020_003335_create_jogo_e_jogador;
mod m20241020_004011_create_jogo;
mod m20241031_011703_cria_apuracao;
mod m20241117_150055_lista_extra;
mod m20241123_213837_imagens;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20241020_003335_create_jogo_e_jogador::Migration),
            Box::new(m20241020_004011_create_jogo::Migration),
            Box::new(m20241031_011703_cria_apuracao::Migration),
            Box::new(m20241117_150055_lista_extra::Migration),
            Box::new(m20241123_213837_imagens::Migration),
        ]
    }
}
