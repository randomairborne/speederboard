use std::sync::Arc;

use axum::extract::{Path, Query, State};

use crate::{
    id::{CategoryMarker, Id},
    model::{Category, DateSort, Game, Permissions, ResolvedRun, RunStatus, User},
    template::BaseRenderInfo,
    util::game_n_member,
    AppState, Error, HandlerResult,
};

const MOD_FEED_PER_PAGE: usize = 100;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ModFeedQuery {
    #[serde(default = "crate::util::return_0_usize")]
    pub page: usize,
    #[serde(default = "crate::util::return_false")]
    pub newest: bool,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct ModFeedPage {
    has_next: bool,
    submissions: Vec<ResolvedRun>,
    category: Option<Category>,
    game: Arc<Game>,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

pub async fn game_feed(
    State(state): State<AppState>,
    base: BaseRenderInfo,
    user: User,

    Path(game_slug): Path<String>,
    Query(query): Query<ModFeedQuery>,
) -> HandlerResult {
    feed_maybe_cat(&state, base, game_slug, None, user, query).await
}

pub async fn category_feed(
    State(state): State<AppState>,
    base: BaseRenderInfo,
    user: User,

    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    Query(query): Query<ModFeedQuery>,
) -> HandlerResult {
    feed_maybe_cat(&state, base, game_slug, Some(category_id), user, query).await
}

async fn feed_maybe_cat(
    state: &AppState,
    base: BaseRenderInfo,
    game_slug: String,
    maybe_category_id: Option<Id<CategoryMarker>>,
    user: User,
    query: ModFeedQuery,
) -> HandlerResult {
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
    let ctx = ModFeedPage {
        base,
        has_next: leaderboard.has_next(),
        category,
        submissions: leaderboard.resolveds(),
        game,
    };
    state.render("moderation_feed.jinja", ctx)
}
