use super::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "game_state")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub game_id: Uuid,
    pub input_id: Uuid,
    #[sea_orm(column_type = "Json")]
    pub data: Value,
    pub created_at: DateTime<Utc>,
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