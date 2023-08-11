use crate::{
    model::User,
    template::BaseRenderInfo,
    util::{AUTHTOKEN_COOKIE, AUTHTOKEN_TTL},
    AppState, Error,
};
use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use rand::distributions::DistString;
use redis::AsyncCommands;
use tera::Context;

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Serialize)]
pub struct LoginPage<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
    incorrect: bool,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Serialize)]
pub struct LoginForm<'a> {
    username: String,
    email: String,
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Deserialize)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Deserialize)]
pub struct LoginQuery {
    pub incorrect: Option<bool>,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
) -> Result<Html<String>, Error> {
    let ctx = LoginPage {
        core: state.base_context(),
        incorrect: query.incorrect.unwrap_or(false),
    };
    let context_ser = Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("login.jinja", &context_ser)?))
}

pub async fn post(
    State(state): State<AppState>,
    cookies: CookieJar,
    Form(form): Form<LoginFormData>,
) -> Result<(CookieJar, Redirect), Error> {
    let Ok(user) = User::from_db(&state, &state.postgres, form.email, form.password).await? else {
        return Ok((cookies, Redirect::to("?incorrect=true")));
    };
    let token = rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    state
        .redis
        .get()
        .await?
        .set_ex(format!("token:user:{token}"), user.id.get(), AUTHTOKEN_TTL)
        .await?;
    state
        .redis
        .get()
        .await?
        .set_ex(
            format!("user:{}", user.id),
            serde_json::to_string(&user)?,
            AUTHTOKEN_TTL,
        )
        .await?;
    Ok((
        cookies.add(Cookie::new(AUTHTOKEN_COOKIE, token)),
        Redirect::to(&state.config.root_url),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    cookies: CookieJar,
) -> Result<(CookieJar, Redirect), Error> {
    let Some(token) = cookies.get(AUTHTOKEN_COOKIE).map(Cookie::value) else {
        return Ok((cookies, Redirect::to(&state.config.root_url)));
    };
    let maybe_id: Option<String> = state
        .redis
        .get()
        .await?
        .get_del(format!("token:user:{token}"))
        .await?;
    let Some(id) = maybe_id else {
        return Ok((cookies, Redirect::to(&state.config.root_url)));
    };
    state.redis.get().await?.del(format!("user:{id}")).await?;
    Ok((
        cookies.remove(Cookie::named(AUTHTOKEN_COOKIE)),
        Redirect::to(&state.config.root_url),
    ))
}
