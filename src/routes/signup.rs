use crate::{
    model::User,
    template::BaseRenderInfo,
    util::{ValidatedForm, AUTHTOKEN_COOKIE, AUTHTOKEN_TTL},
    AppState, Error,
};
use axum::{
    extract::State,
    response::{Html, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use rand::distributions::DistString;
use redis::AsyncCommands;
use tera::Context;

#[derive(serde::Serialize)]
pub struct SignUpPage {
    #[serde(flatten)]
    core: BaseRenderInfo,
}

#[derive(serde::Serialize)]
pub struct SignUpForm {
    username: String,
    email: String,
    #[serde(flatten)]
    core: BaseRenderInfo,
}

#[derive(serde::Deserialize, garde::Validate)]
pub struct SignUpFormData {
    #[garde(email, length(min = crate::util::MIN_EMAIL_LEN, max = crate::util::MAX_EMAIL_LEN))]
    email: String,
    #[garde(length(min = crate::util::MIN_USERNAME_LEN, max = crate::util::MAX_USERNAME_LEN))]
    username: String,
    #[garde(length(min = crate::util::MIN_PASSWORD_LEN))]
    password: String,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    core: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    let ctx = SignUpPage { core };
    let context_ser = Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("signup.jinja", &context_ser)?))
}

pub async fn post(
    State(state): State<AppState>,
    cookies: CookieJar,
    ValidatedForm(form): ValidatedForm<SignUpFormData>,
) -> Result<(CookieJar, Redirect), Error> {
    let password_hash = state
        .spawn_rayon(move |state| {
            crate::util::hash_password(form.password.as_bytes(), &state.argon)
        })
        .await??;
    let user = query_as!(
        User,
        "INSERT INTO users
        (username, email, password, has_stylesheet,
            pfp_ext, banner_ext, biography, admin, created_at)
        VALUES ($1, $2, $3, false, NULL, NULL, '', false, NOW())
        RETURNING id, username, has_stylesheet, pfp_ext,
        banner_ext, biography, admin, created_at",
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
