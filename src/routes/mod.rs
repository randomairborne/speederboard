use crate::{AppState, Error};
use axum::{extract::State, http::StatusCode, response::Html};

pub mod index;
pub mod login;
pub mod signup;
pub mod user;

#[allow(clippy::unused_async)]
pub async fn notfound_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Html<String>), Error> {
    notfound(state)
}

pub fn notfound(state: AppState) -> Result<(StatusCode, Html<String>), Error> {
    let context_ser = tera::Context::from_serialize(state.base_context())?;
    Ok((
        StatusCode::NOT_FOUND,
        Html(state.tera.render("404.jinja", &context_ser)?),
    ))
}
