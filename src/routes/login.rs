use crate::{
    model::User,
    template::BaseRenderInfo,
    util::{ValidatedForm, AUTHTOKEN_COOKIE, AUTHTOKEN_TTL},
    AppState, Error, HandlerResult,
};
use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use rand::distributions::DistString;
use redis::AsyncCommands;

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Serialize, Debug, Clone)]
pub struct LoginPage {
    #[serde(flatten)]
    core: BaseRenderInfo,
    incorrect: bool,
    return_to: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Serialize, Debug, Clone)]
pub struct LoginForm {
    username: String,
    email: String,
    #[serde(flatten)]
    core: BaseRenderInfo,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct LoginFormData {
    #[garde(email, length(min = crate::util::MIN_EMAIL_LEN, max = crate::util::MAX_EMAIL_LEN))]
    pub email: String,
    #[garde(length(min = crate::util::MIN_PASSWORD_LEN))]
    pub password: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Deserialize, Debug, Clone)]
pub struct LoginQuery {
    #[serde(default = "crate::util::default_return_to")]
    pub return_to: String,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
    core: BaseRenderInfo,
) -> HandlerResult {
    let ctx = LoginPage {
        core,
        return_to: query.return_to,
        incorrect: false,
    };
    state.render("login.jinja", ctx)
}

pub async fn post(
    State(state): State<AppState>,
    cookies: CookieJar,
    core: BaseRenderInfo,
    Query(query): Query<LoginQuery>,
    ValidatedForm(form): ValidatedForm<LoginFormData>,
) -> Result<Result<(CookieJar, Redirect), HandlerResult>, Error> {
    let Ok(user) = User::from_db_auth(&state, &state.postgres, form.email, form.password).await?
    else {
        let ctx = LoginPage {
            core,
            return_to: query.return_to,
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
        Redirect::to(&format!("{}{}", state.config.root_url, query.return_to)),
    )))
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
