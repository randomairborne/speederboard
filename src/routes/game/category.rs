use axum::{
    extract::{Path, State},
    response::Html,
};
use tera::Context;

use crate::{
    id::{CategoryMarker, GameMarker, Id},
    model::{Category, Game, ResolvedRunRef, RunStatus, User, MiniCategory},
    template::BaseRenderInfo,
    util::opt_user,
    AppState, Error,
};

#[derive(serde::Serialize)]
pub struct GetGameContext<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo,
    categories: Vec<MiniCategory>,
    category: &'a Category,
    runs: Vec<ResolvedRunRef<'a>>,
    game: &'a Game,
}

pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    core: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    get_game_category(&state, core, game_slug, Some(category_id)).await
}

pub(super) async fn get_game_category(
    state: &AppState,
    core: BaseRenderInfo,
    game_slug: String,
    maybe_category_id: Option<Id<CategoryMarker>>,
) -> Result<Html<String>, Error> {
    let game = Game::from_db_slug(state, &game_slug).await?;
    let category_id = maybe_category_id.unwrap_or(game.default_category);
    let state2 = state.clone();
    let spawned_getcats = tokio::spawn(async move {
        query_as!(
            MiniCategory,
            "SELECT name, id, game, scoreboard
            FROM categories WHERE game = $1",
            game.id.get()
        )
        .fetch_all(&state2.postgres)
        .await
    });
    let category = query_as!(
        Category,
        "SELECT * FROM categories WHERE id = $1",
        category_id.get()
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let runs: Vec<ResolvedRunRef> = if category.scoreboard {
        get_scoreboard(state, &game, &category).await?
    } else {
        get_speedrun(state, &game, &category).await?
    };
    let categories = spawned_getcats.await??;
    let get_game_ctx = GetGameContext {
        core,
        categories,
        category: &category,
        runs,
        game: &game,
    };
    let ctx = Context::from_serialize(get_game_ctx)?;
    Ok(Html(state.tera.render("category.jinja", &ctx)?))
}