use sea_orm_migration::{prelude::*, sea_orm::Schema};


#[derive(DeriveIden)]
enum GameType {
    Table,
    Id,
    Name,
    Description,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Username,
    PasswordHash,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Game {
    Table,
    Id,
    GameType,
    CreatedAt,
}

#[derive(DeriveIden)]
enum GameInput {
    Table,
    Id,
    GameId,
    Username,
    Data,
    CreatedAt,
}

#[derive(DeriveIden)]
enum GameState {
    Table,
    Id,
    GameId,
    InputId,
    Data,
    CreatedAt
}

#[derive(DeriveIden)]
enum GameParticipant {
    Table,
    Id,
    GameId,
    Username,
    JoinedAt,
}


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        
        manager
            .create_table(
                Table::create()
                    .table(GameType::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GameType::Id).integer().not_null().primary_key().auto_increment())
                    .col(ColumnDef::new(GameType::Name).string().not_null())
                    .col(ColumnDef::new(GameType::Description).string().not_null())
                    .to_owned(),
            )
            .await?;
        
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Username).string().not_null().primary_key())
                    .col(ColumnDef::new(User::PasswordHash).string().not_null())
                    .col(ColumnDef::new(User::CreatedAt).timestamp_with_time_zone())
                    .to_owned()
            )
            .await?;
        
        manager
            .create_table(
                Table::create()
                    .table(Game::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Game::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Game::GameType).string().not_null())
                    .col(ColumnDef::new(Game::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned()
            )
            .await?;


        manager
            .create_table(
                Table::create()
                    .table(GameInput::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GameInput::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(GameInput::GameId).uuid().not_null())
                    .col(ColumnDef::new(GameInput::Username).string().not_null())
                    .col(ColumnDef::new(GameInput::Data).json().not_null())
                    .col(ColumnDef::new(GameInput::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .from(GameInput::Table, GameInput::GameId)
                        .to(Game::Table, Game::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                        .from(GameInput::Table, GameInput::Username)
                        .to(User::Table, User::Username)
                    )
                    .to_owned()
            )
        .await?;

        manager.create_table(
            Table::create()
                .table(GameState::Table)
                .if_not_exists()
                .col(ColumnDef::new(GameState::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(GameState::GameId).uuid().not_null())
                .col(ColumnDef::new(GameState::InputId).uuid().not_null().unique_key())
                .col(ColumnDef::new(GameState::Data).json().not_null())
                .col(ColumnDef::new(GameState::CreatedAt).timestamp_with_time_zone().not_null())
                .foreign_key(
                    ForeignKey::create()
                    .from(GameState::Table, GameState::GameId)
                    .to(Game::Table, Game::Id)
                )
                .foreign_key(
                    ForeignKey::create()
                    .from(GameState::Table, GameState::InputId)
                    .to(GameInput::Table, GameInput::Id)
                )
                .to_owned()
        ).await?;

        manager.create_table(
            Table::create()
                .table(GameParticipant::Table)
                .if_not_exists()
                .col(ColumnDef::new(GameParticipant::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(GameParticipant::GameId).uuid().not_null())
                .col(ColumnDef::new(GameParticipant::Username).string().not_null())
                .col(ColumnDef::new(GameParticipant::JoinedAt).not_null().timestamp_with_time_zone())
                .foreign_key(
                    ForeignKey::create()
                    .from(GameParticipant::Table, GameParticipant::Username)
                    .to(User::Table, User::Username)
                )
                .foreign_key(
                    ForeignKey::create()
                    .from(GameParticipant::Table, GameParticipant::GameId)
                    .to(Game::Table, Game::Id)
                )
                .to_owned()
        ).await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user_username")
                    .table(User::Table)
                    .col(User::Username)
                    .to_owned()
            ).await?;

        Ok(())
    
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.drop_index(Index::drop().name("idx-user_username").if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GameParticipant::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GameState::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GameInput::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Game::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GameType::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}
