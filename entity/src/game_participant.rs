use super::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "game_participant")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub game_id: Uuid,
    pub username: String,
    pub joined_at: DateTime<Utc>,
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