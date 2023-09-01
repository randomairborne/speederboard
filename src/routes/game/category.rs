use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use tera::Context;

use crate::{
    id::{CategoryMarker, Id},
    model::{Category, Game, MiniCategory, ResolvedRun, RunStatus, SortBy},
    template::BaseRenderInfo,
    AppState, Error,
};

fn ret_0() -> usize {
    0
}

#[derive(serde::Deserialize)]
pub struct GetCategoryQuery {
    #[serde(default = "ret_0")]
    pub page: usize,
}

#[derive(serde::Serialize)]
pub struct GetGameContext {
    #[serde(flatten)]
    core: BaseRenderInfo,
    categories: Vec<MiniCategory>,
    category: Category,
    has_next: bool,
    runs: Vec<ResolvedRun>,
    game: Arc<Game>,
}

pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    Query(query): Query<GetCategoryQuery>,
    core: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    get_game_category(&state, core, game_slug, Some(category_id), query.page).await
}

const RUNS_PER_PAGE: usize = 50;

pub(super) async fn get_game_category(
    state: &AppState,
    core: BaseRenderInfo,
    game_slug: String,
    maybe_category_id: Option<Id<CategoryMarker>>,
    page: usize,
) -> Result<Html<String>, Error> {
    let game = Arc::new(Game::from_db_slug(state, &game_slug).await?);
    let category_id = maybe_category_id.unwrap_or(game.default_category);
    let state2 = state.clone();
    let game_id = game.id.get();
    let spawned_getcats = tokio::spawn(async move {
        query_as!(
            MiniCategory,
            "SELECT name, id, game, scoreboard
            FROM categories WHERE game = $1",
            game_id
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
    let sort_by = if category.scoreboard {
        SortBy::Score
    } else {
        SortBy::Time
    };
    let resolution = ResolvedRun::fetch_leaderboard(
        state,
        game.clone(),
        RunStatus::Verified,
        Some(category.id),
        sort_by,
        RUNS_PER_PAGE,
        page,
    )
    .await?;
    let categories = spawned_getcats.await??;
    let get_game_ctx = GetGameContext {
        core,
        categories,
        category,
        has_next: resolution.has_next(),
        runs: resolution.resolveds(),
        game,
    };
    let ctx = Context::from_serialize(get_game_ctx)?;
    Ok(Html(state.tera.render("category.jinja", &ctx)?))
}
