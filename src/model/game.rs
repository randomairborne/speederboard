use crate::{
    id::{CategoryMarker, GameMarker, Id},
    AppState, Error,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Encode, Hash, PartialEq, Eq, Clone)]
pub struct Game {
    pub id: Id<GameMarker>,
    pub name: String,
    pub slug: String,
    pub url: String,
    pub default_category: Id<CategoryMarker>,
    pub description: String,
    pub has_stylesheet: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_ext: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art_ext: Option<String>,
}

impl Game {
    pub async fn from_db_slug(state: &AppState, slug: &str) -> Result<Self, Error> {
        let Some(game) = query_as!(Game, "SELECT * FROM games WHERE slug = $1", slug)
            .fetch_optional(&state.postgres)
            .await?
        else {
            return Err(Error::NotFound);
        };
        Ok(game)
    }
    pub async fn from_db_id(state: &AppState, id: Id<GameMarker>) -> Result<Self, Error> {
        let game = query_as!(Game, "SELECT * FROM games WHERE id = $1", id.get())
            .fetch_one(&state.postgres)
            .await?;
        Ok(game)
    }
}