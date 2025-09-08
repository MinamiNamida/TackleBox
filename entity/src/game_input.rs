use super::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "game_input")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub game_id: Uuid,
    pub username: String,
    #[sea_orm(column_type = "Json")]
    pub data: Value,
    pub created_at: DateTime<Utc>,
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