use crate::{model::Game, template::BaseRenderInfo, AppState, Error};
use axum::{extract::State, response::Html};
use tera::Context;

#[derive(serde::Serialize)]
struct RootContext {
    games: Vec<Game>,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    base: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    let games = query_as!(Game, "SELECT * FROM games")
        .fetch_all(&state.postgres)
        .await?;
    let context_ser = Context::from_serialize(RootContext { games, base })?;
    Ok(Html(state.tera.render("index.jinja", &context_ser)?))
}
