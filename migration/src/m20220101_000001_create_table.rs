use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Ballot {
    Table,
    Id,
    Players,
    Vote,
    Date,
    Voter,
    FuteId,
    State,
}

#[derive(Iden, EnumIter)]
enum BallotState {
    #[iden = "generated"]
    Generated,
    #[iden = "submitted"]
    Submitted,
    #[iden = "computed"]
    Computed,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ballot::Table)
                    .if_not_exists()
                    .col(pk_auto(Ballot::Id))
                    .col(json(Ballot::Players))
                    .col(json(Ballot::Vote))
                    .col(timestamp(Ballot::Date))
                    .col(string_len(Ballot::Voter, 255))
                    .col(integer(Ballot::FuteId))
                    .col(enumeration(
                        Ballot::State,
                        Alias::new("ballot_state"),
                        BallotState::iter(),
                    ))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Ballot::Table).to_owned())
            .await
    }
}
