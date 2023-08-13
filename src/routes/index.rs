use crate::{template::BaseRenderInfo, AppState, Error};
use axum::{extract::State, response::Html};
use tera::Context;

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    base: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    let context_ser = Context::from_serialize(base)?;
    Ok(Html(state.tera.render("index.jinja", &context_ser)?))
}
