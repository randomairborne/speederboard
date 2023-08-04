use axum::{extract::State, response::Html, Form};
use tera::Context;

use crate::{template::BaseRenderInfo, AppState, Error};

#[derive(serde::Serialize)]
pub struct LoginPage<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
}

#[derive(serde::Serialize)]
pub struct LoginForm<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
}

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    email: String,
    username: String,
    password: String,
}

pub async fn page(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let ctx = LoginPage {
        core: BaseRenderInfo::new(&state.config.root_url),
    };
    let context_ser = Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("login.jinja", &context_ser)?))
}

pub async fn form(
    State(state): State<AppState>,
    Form(form): Form<LoginFormData>,
) -> Result<Html<String>, Error> {
    let ctx = LoginPage {
        core: BaseRenderInfo::new(&state.config.root_url),
    };
    let context_ser = Context::from_serialize(ctx)?;
    Ok(Html(
        state.tera.render("login_success.jinja", &context_ser)?,
    ))
}
