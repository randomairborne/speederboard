use crate::{AppState, Error};
use axum::{extract::State, response::Html};

pub mod login;
pub mod index;
pub mod signup;

#[allow(clippy::unused_async)]
pub async fn notfound(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let context_ser = tera::Context::from_serialize(state.base_context())?;
    Ok(Html(state.tera.render("404.jinja", &context_ser)?))
}
