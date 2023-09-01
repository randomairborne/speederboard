use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::{
    id::{CategoryMarker, Id},
    model::{Game, ResolvedRunRef, Category},
    template::BaseRenderInfo,
    AppState, Error,
};

#[derive(serde::Serialize)]
struct ModFeedContext<'a> {
    submissions: Vec<ResolvedRunRef<'a>>,
    category: Option<&'a Category>,
    runs: Vec<ResolvedRunRef<'a>>,
    game: &'a Game,
    #[serde(flatten)]
    core: BaseRenderInfo
}

pub async fn game_feed(
    State(state): State<AppState>,
    core: BaseRenderInfo,
    Path(game_slug): Path<String>,
) -> Result<Html<String>, Error> {
    feed_maybe_cat(&state, core, game_slug, None).await
}

pub async fn category_feed(
    State(state): State<AppState>,
    core: BaseRenderInfo,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
) -> Result<Html<String>, Error> {
    feed_maybe_cat(&state, core, game_slug, Some(category_id)).await
}

async fn feed_maybe_cat(
    state: &AppState,
    core: BaseRenderInfo,
    game_slug: String,
    maybe_category_id: Option<Id<CategoryMarker>>,
) -> Result<Html<String>, Error> {
    let game = Game::from_db_slug(state, &game_slug).await?;
    let mut category = None;
    let submissions = if let Some(cat_id) = maybe_category_id {
        query!("SELECT * FROM runs WHERE game = $1 AND category = $2 AND status = 0", game.id.get(), cat_id.get()).fetch_all(&state.postgres).await?
    } else {
        query!("").fetch_all(&state.postgres).await?
    };
    let mod_feed_ctx = ModFeedContext {
        core,
        category: category,
        submissions,
        game: &game,
    };
    let ctx = tera::Context::from_serialize(mod_feed_ctx)?;
    Ok(Html(state.tera.render("moderation_feed.jinja", &ctx)?))
}
