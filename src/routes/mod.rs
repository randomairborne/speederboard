use crate::{template::BaseRenderInfo, AppState, Error};
use axum::{extract::State, response::Html};

pub mod login;
pub mod root;
pub mod signup;

#[allow(clippy::unused_async)]
pub async fn notfound(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let bri = BaseRenderInfo::new(&state.config.root_url);
    let context_ser = tera::Context::from_serialize(bri)?;
    Ok(Html(state.tera.render("404.jinja", &context_ser)?))
}
