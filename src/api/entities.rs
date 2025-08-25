use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::NaiveDateTime;


pub mod user {
    use super::*;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "users")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub username: String,
        pub password_hash: String,
        // pub created_at: Option<NaiveDateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

}

pub use {
    user::Column as UserCol, 
    user::Entity as User,
    user::ActiveModel as UserAclModel,
    user::Model as UserModel,
};


pub mod game_type {
    use super::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "game_types")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[sea_orm(auto_increment = true)]
        pub id: i32,
        pub name: String,
        pub description: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {
        Game,
    }

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            match self {
                Self::Game => Entity::has_many(Game).into(),
            }
        }
    }

    impl Related<Game> for Entity {
        fn to() -> RelationDef {
            Relation::Game.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub use {
    game_type::Column as GameTypeCol,
    game_type::Entity as GameType,
    game_type::ActiveModel as GameTypeAclModel,
    game_type::Model as GameTypeModel,
};


pub mod game {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "games")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[sea_orm(auto_increment = true)]
        pub id: Uuid,
        pub status: Option<String>,
        pub game_type_id: i32,
        pub created_at: Option<NaiveDateTime>,
        pub finished_at: Option<NaiveDateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {
        GameType,
        GameState,
        GameInput,
        GameParticipant,
    }

    impl Related<GameType> for Entity {
        fn to() -> RelationDef {
            Relation::GameType.def()
        }
    }

    impl Related<GameState> for Entity {
        fn to() -> RelationDef {
            Relation::GameState.def()
        }
    }

    impl Related<GameInput> for Entity {
        fn to() -> RelationDef {
            Relation::GameState.def()
        }
    }

    impl Related<GameParticipant> for Entity {
        fn to() -> RelationDef {
            Relation::GameParticipant.def()
        }
    }

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            match self {
                Self::GameType => Entity::belongs_to(GameType)
                    .from(Column::GameTypeId)
                    .to(GameTypeCol::Id)
                    .into(),
                Self::GameState => Entity::has_many(GameState).into(),
                Self::GameInput => Entity::has_many(GameInput).into(),
                Self::GameParticipant => Entity::has_many(GameParticipant).into(),
            }
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub use {
    game::Column as GameCol,
    game::Entity as Game,
    game::ActiveModel as GameAclModel,
    game::Model as GameModel,
};



pub mod game_input {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "game_inputs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[sea_orm(auto_increment = true)]
        pub id: Uuid,
        pub game_id: Uuid,
        pub username: String,
        pub turn_number: i32,
        #[sea_orm(column_type = "Json")]
        pub input_data: Value,
        pub created_at: Option<NaiveDateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {
        Game,
        User,
        GameState,
    }

    impl Related<GameState> for Entity {
        fn to() -> RelationDef {
            Relation::GameState.def()
        }
    }

    impl Related<User> for Entity {
        fn to() -> RelationDef {
            Relation::User.def()
        }
    }
    
    impl Related<Game> for Entity {
        fn to() -> RelationDef {
            Relation::Game.def()
        }
    }

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            match self {
                Self::Game => Entity::belongs_to(Game)
                    .from(Column::GameId)
                    .to(game::Column::Id)
                    .into(),
                Self::User => Entity::belongs_to(User)
                    .from(Column::Username)
                    .to(user::Column::Username)
                    .into(),
                Self::GameState => Entity::has_one(GameState).into(),
            }
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    
}

pub use {
    game_input::Column as GameInputCol,
    game_input::Entity as GameInput,
    game_input::ActiveModel as GameInputAclModel,
    game_input::Model as GameInputModel,
};

pub mod game_state {
    use super::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "game_states")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[sea_orm(auto_increment = true)]
        pub id: Uuid,
        pub game_id: Uuid,
        pub input_id: Uuid, // UNIQUE + FK -> game_inputs.id to enforce 1:1
        pub turn_number: i32,
        #[sea_orm(column_type = "Json")]
        pub state_data: Value,
        pub created_at: Option<NaiveDateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {
        Game,
        GameInput,
    }

    impl Related<GameInput> for Entity {
        fn to() -> RelationDef {
            Relation::GameInput.def()
        }
    }

    impl Related<Game> for Entity {
        fn to() -> RelationDef {
            Relation::Game.def()
        }
    }

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            match self {
                Self::Game => Entity::belongs_to(game::Entity)
                    .from(Column::GameId)
                    .to(game::Column::Id)
                    .into(),
                Self::GameInput => Entity::belongs_to(game_input::Entity)
                    .from(Column::InputId)
                    .to(game_input::Column::Id)
                    .into(),
            }
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub use {
    game_state::Column as GameStateCol,
    game_state::Entity as GameState,
    game_state::ActiveModel as GameStateAclModel,
    game_state::Model as GameStateModel,
};


pub mod game_participant {
    use super::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "game_participants")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[sea_orm(auto_increment = true)]
        pub id: Uuid,
        pub game_id: Uuid,
        pub username: String,
        pub role: Option<String>,
        pub joined_at: Option<NaiveDateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {
        Game,
        User,
    }

    impl Related<Game> for Entity {
        fn to() -> RelationDef {
            Relation::Game.def()
        }
    }

    impl Related<User> for Entity {
        fn to() -> RelationDef {
            Relation::User.def()
        }
    }

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            match self {
                Self::Game => Entity::belongs_to(game::Entity)
                    .from(Column::GameId)
                    .to(game::Column::Id)
                    .into(),
                Self::User => Entity::belongs_to(user::Entity)
                    .from(Column::Username)
                    .to(user::Column::Username)
                    .into(),
            }
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub use {
    game_participant::Column as GameParticipantCol,
    game_participant::Entity as GameParticipant,
    game_participant::ActiveModel as GameParticipantAclModel,
    game_participant::Model as GameParticipantModel,
};