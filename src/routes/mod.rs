use crate::{template::BaseRenderInfo, AppState, Error};
use axum::{extract::State, http::StatusCode, response::Html};

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
) -> Result<(StatusCode, Html<String>), Error> {
    notfound(&state, core)
}

pub fn notfound(
    state: &AppState,
    core: BaseRenderInfo,
) -> Result<(StatusCode, Html<String>), Error> {
    let context_ser = tera::Context::from_serialize(core)?;
    Ok((
        StatusCode::NOT_FOUND,
        Html(state.tera.render("404.jinja", &context_ser)?),
    ))
}
