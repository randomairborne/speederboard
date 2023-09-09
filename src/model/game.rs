use redis::AsyncCommands;

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
        match Self::get_game_slug_redis(state, slug).await {
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
        if let Err(source) = Self::set_redis_game_cache(state, slug, &game).await {
            error!(?source, "failed to set game redis cache");
        }
        Ok(game)
    }

    async fn get_game_slug_redis(state: &AppState, slug: &str) -> Result<Option<Self>, Error> {
        let maybe_game_str: Option<String> =
            state.redis.get().await?.get(format!("game:{slug}")).await?;
        let Some(game_str) = maybe_game_str else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_str(&game_str)?))
    }

    async fn set_redis_game_cache(state: &AppState, slug: &str, game: &Game) -> Result<(), Error> {
        let game_str = serde_json::to_string(game)?;
        state
            .redis
            .get()
            .await?
            .set_ex(format!("game:{slug}"), game_str, 600)
            .await?;
        Ok(())
    }
}
