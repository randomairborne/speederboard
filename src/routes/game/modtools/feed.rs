use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::Html,
};

use crate::{
    id::{CategoryMarker, Id},
    model::{Category, DateSort, Game, Permissions, ResolvedRun, RunStatus, User},
    template::BaseRenderInfo,
    util::game_n_member,
    AppState, Error,
};

const MOD_FEED_PER_PAGE: usize = 100;

fn ret_0() -> usize {
    0
}

fn ret_false() -> bool {
    false
}

#[derive(serde::Deserialize)]
pub struct ModFeedQuery {
    #[serde(default = "ret_0")]
    pub page: usize,
    #[serde(default = "ret_false")]
    pub newest: bool,
}

#[derive(serde::Serialize)]
struct ModFeedContext {
    has_next: bool,
    submissions: Vec<ResolvedRun>,
    category: Option<Category>,
    game: Arc<Game>,
    #[serde(flatten)]
    core: BaseRenderInfo,
}

pub async fn game_feed(
    State(state): State<AppState>,
    core: BaseRenderInfo,
    user: User,

    Path(game_slug): Path<String>,
    Query(query): Query<ModFeedQuery>,
) -> Result<Html<String>, Error> {
    feed_maybe_cat(&state, core, game_slug, None, user, query).await
}

pub async fn category_feed(
    State(state): State<AppState>,
    core: BaseRenderInfo,
    user: User,

    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    Query(query): Query<ModFeedQuery>,
) -> Result<Html<String>, Error> {
    feed_maybe_cat(&state, core, game_slug, Some(category_id), user, query).await
}

async fn feed_maybe_cat(
    state: &AppState,
    core: BaseRenderInfo,
    game_slug: String,
    maybe_category_id: Option<Id<CategoryMarker>>,
    user: User,
    query: ModFeedQuery,
) -> Result<Html<String>, Error> {
    let (game, member) = game_n_member(state, user, &game_slug).await?;
    if !member.perms.contains(Permissions::VERIFY_RUNS) {
        return Err(Error::InsufficientPermissions);
    }
    let game = Arc::new(game);
    let sort_direction = if query.newest {
        DateSort::Newest
    } else {
        DateSort::Oldest
    };
    let state2 = state.clone();
    let mut maybe_task = None;
    if let Some(cat_id) = maybe_category_id {
        maybe_task = Some(tokio::spawn(Category::from_db(state2, cat_id)));
    }
    let leaderboard = ResolvedRun::fetch_leaderboard(
        state,
        game.clone(),
        RunStatus::Pending,
        maybe_category_id,
        crate::model::SortBy::SubmissionDate(sort_direction),
        MOD_FEED_PER_PAGE,
        query.page,
    )
    .await?;
    let category = if let Some(task) = maybe_task {
        Some(task.await??)
    } else {
        None
    };
    let mod_feed_ctx = ModFeedContext {
        core,
        has_next: leaderboard.has_next(),
        category,
        submissions: leaderboard.resolveds(),
        game,
    };
    let ctx = tera::Context::from_serialize(mod_feed_ctx)?;
    Ok(Html(state.tera.render("moderation_feed.jinja", &ctx)?))
}
