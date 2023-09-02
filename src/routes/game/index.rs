use axum::extract::{Path, Query, State};

use crate::{template::BaseRenderInfo, AppState, HandlerResult};

use super::category::{get_game_category, GetCategoryQuery};

pub async fn get(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    Query(query): Query<GetCategoryQuery>,
    core: BaseRenderInfo,
) -> HandlerResult {
    get_game_category(&state, core, game_slug, None, query.page).await
}
