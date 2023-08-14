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
}

impl Category {
    pub async fn from_db(
        state: &AppState,
        id: Id<CategoryMarker>,
    ) -> Result<Option<Category>, Error> {
        Ok(query_as!(
            Category,
            "SELECT id, game, name, description, rules, scoreboard FROM categories WHERE id = $1",
            id.get()
        )
        .fetch_optional(&state.postgres)
        .await?)
    }
}
