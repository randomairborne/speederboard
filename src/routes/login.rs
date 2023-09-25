use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use rand::distributions::DistString;
use redis::AsyncCommands;

use crate::{
    model::User,
    template::BaseRenderInfo,
    util::{ValidatedForm, AUTHTOKEN_COOKIE, AUTHTOKEN_TTL},
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct LoginPage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    incorrect: bool,
    return_to: String,
}

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct LoginForm {
    #[garde(email, length(min = crate::util::MIN_EMAIL_LEN, max = crate::util::MAX_EMAIL_LEN))]
    email: String,
    #[garde(length(min = crate::util::MIN_PASSWORD_LEN))]
    password: String,
    #[garde(skip)]
    return_to: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct LoginQuery {
    #[serde(default = "crate::util::default_return_to")]
    return_to: String,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
    base: BaseRenderInfo,
) -> HandlerResult {
    let ctx = LoginPage {
        base,
        return_to: query.return_to,
        incorrect: false,
    };
    state.render("login.jinja", ctx)
}

pub async fn post(
    State(state): State<AppState>,
    cookies: CookieJar,
    base: BaseRenderInfo,
    ValidatedForm(form): ValidatedForm<LoginForm>,
) -> Result<Result<(CookieJar, Redirect), HandlerResult>, Error> {
    let Ok(user) = User::from_db_auth(&state, &state.postgres, form.email, form.password).await?
    else {
        let ctx = LoginPage {
            base,
            return_to: form.return_to,
            incorrect: true,
        };
        return Ok(Err(state.render("login.jinja", ctx)));
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
    Ok(Ok((
        cookies.add(Cookie::new(AUTHTOKEN_COOKIE, token)),
        state.redirect(form.return_to),
    )))
}

pub async fn logout(
    State(state): State<AppState>,
    cookies: CookieJar,
) -> Result<(CookieJar, Redirect), Error> {
    let Some(token) = cookies.get(AUTHTOKEN_COOKIE).map(Cookie::value) else {
        return Ok((cookies, state.redirect("/")));
    };
    let maybe_id: Option<String> = state
        .redis
        .get()
        .await?
        .get_del(format!("token:user:{token}"))
        .await?;
    let Some(id) = maybe_id else {
        return Ok((cookies, state.redirect("/")));
    };
    state.redis.get().await?.del(format!("user:{id}")).await?;
    Ok((
        cookies.remove(Cookie::named(AUTHTOKEN_COOKIE)),
        state.redirect("/"),
    ))
}
