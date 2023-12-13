use crate::{
    id::{CategoryMarker, GameMarker, Id},
    AppState, Error,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Encode, Hash, PartialEq, Eq, Clone)]
pub struct Category {
    pub id: Id<CategoryMarker>,
    pub game: Id<GameMarker>,
    pub name: String,
    pub description: String,
    pub rules: String,
    pub scoreboard: bool,
    pub flags: i64,
}

impl Category {
    pub async fn from_db(state: &AppState, id: Id<CategoryMarker>) -> Result<Category, Error> {
        query_as!(
            Category,
            "SELECT id, game, name, description, rules, scoreboard, flags FROM categories WHERE id = $1",
            id.get()
        )
        .fetch_optional(&state.postgres)
        .await?
        .ok_or(Error::NotFound)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Encode, Hash, PartialEq, Eq, Clone)]
pub struct MiniCategory {
    pub name: String,
    pub id: Id<CategoryMarker>,
    pub game: Id<GameMarker>,
    pub scoreboard: bool,
    pub flags: i64,
}
