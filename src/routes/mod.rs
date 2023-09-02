use crate::{template::BaseRenderInfo, AppState, HandlerResult};
use axum::{extract::State, http::StatusCode};

pub mod admin;
pub mod game;
pub mod index;
pub mod login;
pub mod settings;
pub mod signup;
pub mod user;

#[allow(clippy::unused_async)]
pub async fn notfound_handler(
    State(state): State<AppState>,
    core: BaseRenderInfo,
) -> (StatusCode, HandlerResult) {
    notfound(&state, core)
}

pub fn notfound(state: &AppState, core: BaseRenderInfo) -> (StatusCode, HandlerResult) {
    (StatusCode::NOT_FOUND, state.render("404.jinja", core))
}
