use crate::{user::FrontendUser, AppState, Error};
use axum::{extract::State, response::Html};
use tera::Context;

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    user: Option<FrontendUser>,
) -> Result<Html<String>, Error> {
    let mut base = state.base_context();
    base.logged_in_user = user;
    let context_ser = Context::from_serialize(base)?;
    Ok(Html(state.tera.render("index.jinja", &context_ser)?))
}
