use axum::extract::State;

use crate::{model::Game, template::BaseRenderInfo, AppState, HandlerResult};

#[derive(serde::Serialize, Debug, Clone)]
pub struct RootPage {
    games: Vec<Game>,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

#[allow(clippy::unused_async)]
pub async fn get(State(state): State<AppState>, base: BaseRenderInfo) -> HandlerResult {
    let games = query_as!(Game, "SELECT * FROM games")
        .fetch_all(&state.postgres)
        .await?;
    let ctx = RootPage { games, base };
    state.render("index.jinja", ctx)
}
