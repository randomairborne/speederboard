use crate::{
    template::BaseRenderInfo,
    user::{User, TOKEN_COOKIE},
    AppState, Error,
};
use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use rand::distributions::DistString;
use redis::AsyncCommands;
use tera::Context;

#[derive(serde::Serialize)]
pub struct SignUpPage<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
}

#[derive(serde::Serialize)]
pub struct SignUpForm<'a> {
    username: String,
    email: String,
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
}

#[derive(serde::Deserialize)]
pub struct SignUpFormData {
    email: String,
    username: String,
    password: String,
}

#[allow(clippy::unused_async)]
pub async fn get(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let ctx = SignUpPage {
        core: state.base_context(),
    };
    let context_ser = Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("signup.jinja", &context_ser)?))
}

pub async fn post(
    State(state): State<AppState>,
    cookies: CookieJar,
    Form(form): Form<SignUpFormData>,
) -> Result<(CookieJar, Redirect), Error> {
    let password_hash = state
        .spawn_rayon(move |state| {
            crate::utils::hash_password(form.password.as_bytes(), &state.argon)
        })
        .await??;
    let user = query_as!(
        User,
        "INSERT INTO users
        (username, email, password, has_stylesheet, pfp_ext, banner_ext, biography)
        VALUES ($1, $2, $3, false, NULL, NULL, '')
        RETURNING id, username, has_stylesheet, pfp_ext, banner_ext, biography",
        form.username,
        form.email,
        password_hash.to_string()
    )
    .fetch_one(&state.postgres)
    .await?;
    let token = rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    state
        .redis
        .get()
        .await?
        .set_ex(
            format!("token:user:{token}"),
            serde_json::to_string(&user)?,
            86_400,
        )
        .await?;
    Ok((
        cookies.add(Cookie::new(TOKEN_COOKIE, token)),
        Redirect::to(&state.config.root_url),
    ))
}
