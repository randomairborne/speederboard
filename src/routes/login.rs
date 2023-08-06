use crate::{
    template::BaseRenderInfo,
    user::{User, TOKEN_COOKIE},
    AppState, Error,
};
use argon2::{PasswordHash, PasswordVerifier};
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
    email: String,
    password: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Deserialize)]
pub struct LoginQuery {
    incorrect: Option<bool>,
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
    let Some(record) = query!(
        "SELECT * FROM users WHERE email = $1",
        form.email
    )
    .fetch_optional(&state.postgres)
    .await? else {
        return Ok((cookies, Redirect::to("?incorrect=true")));
    };
    let user = User {
        id: record.id.into(),
        username: record.username,
        has_stylesheet: record.has_stylesheet,
        pfp_ext: record.pfp_ext,
        banner_ext: record.banner_ext,
        biography: record.biography,
    };
    let password_result = state
        .spawn_rayon(move |state| {
            let hash = PasswordHash::new(&record.password)?;
            state.argon.verify_password(form.password.as_ref(), &hash)
        })
        .await?;
    if let Err(argon2::password_hash::Error::Password) = password_result {
        return Ok((cookies, Redirect::to("?incorrect=true")));
    }
    // this looks a little weird! but we do this because if there's an error verifying
    // a password, we want to report it, but differently then if the password is *wrong*
    password_result?;
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
