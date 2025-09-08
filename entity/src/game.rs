use super::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "game")]
pub struct Model {
    #[sea_orm(primary_key)]
    // #[sea_orm(auto_increment = true)]
    pub id: Uuid,
    pub game_type: String,
    // pub log: Vec<Value>,
    pub created_at: DateTime<Utc>,
    // pub finished_at: DateTime,
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
                .from(Column::GameType)
                .to(GameTypeCol::Name)
                .into(),
            Self::GameState => Entity::has_many(GameState).into(),
            Self::GameInput => Entity::has_many(GameInput).into(),
            Self::GameParticipant => Entity::has_many(GameParticipant).into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}