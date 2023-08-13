use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::{template::BaseRenderInfo, AppState, Error};

use super::category::get_game_category;

pub async fn get(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    core: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    get_game_category(&state, core, game_slug, None).await
}
