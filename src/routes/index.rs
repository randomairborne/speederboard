use crate::{model::Game, template::BaseRenderInfo, AppState, HandlerResult};
use axum::extract::State;

#[derive(serde::Serialize)]
struct RootContext {
    games: Vec<Game>,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

#[allow(clippy::unused_async)]
pub async fn get(State(state): State<AppState>, base: BaseRenderInfo) -> HandlerResult {
    let games = query_as!(Game, "SELECT * FROM games")
        .fetch_all(&state.postgres)
        .await?;
    let ctx = RootContext { games, base };
    state.render("index.jinja", ctx)
}
