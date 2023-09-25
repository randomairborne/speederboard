use axum::{
    extract::{OriginalUri, State},
    http::StatusCode,
};

use crate::{template::BaseRenderInfo, AppState, HandlerResult};

pub mod admin;
pub mod forum;
pub mod game;
pub mod index;
pub mod login;
pub mod settings;
pub mod signup;
pub mod user;

#[derive(serde::Serialize, Clone, Debug)]
pub struct NotFoundPage {
    page: String,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

#[allow(clippy::unused_async)]
pub async fn notfound_handler(
    State(state): State<AppState>,
    base: BaseRenderInfo,
    uri: OriginalUri,
) -> (StatusCode, HandlerResult) {
    notfound(&state, base, uri.to_string())
}

pub fn notfound(
    state: &AppState,
    base: BaseRenderInfo,
    page: String,
) -> (StatusCode, HandlerResult) {
    let page = NotFoundPage { page, base };
    (StatusCode::NOT_FOUND, state.render("404.jinja", page))
}
