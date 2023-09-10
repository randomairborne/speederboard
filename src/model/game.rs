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
    pub flags: i64,
}

impl Game {
    pub async fn from_db_slug(state: &AppState, slug: &str) -> Result<Self, Error> {
        match crate::util::get_redis_object(state, format!("game:{slug}")).await {
            Ok(Some(game)) => return Ok(game),
            Ok(None) => trace!(slug, "did not find game slug in redis cache"),
            Err(source) => error!(
                ?source,
                slug, "an error occured trying to find game slug in redis cache"
            ),
        };
        let Some(game) = query_as!(Game, "SELECT * FROM games WHERE slug = $1", slug)
            .fetch_optional(&state.postgres)
            .await?
        else {
            return Err(Error::NotFound);
        };
        if let Err(source) = crate::util::set_redis_object(state, slug, &game, 600).await {
            error!(?source, "failed to set game redis cache");
        }
        Ok(game)
    }
}
