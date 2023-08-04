use axum::{extract::State, response::{Html, Redirect}, Form, http::StatusCode};
use rand::distributions::DistString;
use tera::Context;
use tower_cookies::{Cookie, Cookies};

use crate::{template::BaseRenderInfo, AppState, Error};

#[derive(serde::Serialize)]
pub struct LoginPage<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
}

#[derive(serde::Serialize)]
pub struct LoginForm<'a> {
    username: String,
    email: String,
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
}

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    email: String,
    password: String,
}

#[allow(clippy::unused_async)]
pub async fn page(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let ctx = LoginPage {
        core: BaseRenderInfo::new(&state.config.root_url),
    };
    let context_ser = Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("signup.jinja", &context_ser)?))
}

pub async fn form(
    State(state): State<AppState>,
    Form(form): Form<LoginFormData>,
    cookies: Cookies
) -> Result<Redirect, Error> {
    query!("SELECT");
    let token = rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    cookies.add(Cookie::new("auth", format!("SPDBRD.{token}")));
    Ok(Redirect::to(&state.config.root_url))
}
