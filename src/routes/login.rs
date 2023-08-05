use crate::user::AuthUser;
use crate::{template::BaseRenderInfo, AppState, Error};
use argon2::{PasswordHash, PasswordVerifier};
use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use rand::distributions::DistString;
use tera::Context;

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Serialize)]
pub struct LoginPage<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
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

#[allow(clippy::unused_async)]
pub async fn page(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let ctx = LoginPage {
        core: BaseRenderInfo::new(&state.config.root_url),
    };
    let context_ser = Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("login.jinja", &context_ser)?))
}

pub async fn form(
    State(state): State<AppState>,
    cookies: CookieJar,
    Form(form): Form<LoginFormData>,
) -> Result<(CookieJar, Redirect), Error> {
    let output = query_as!(AuthUser, "SELECT * FROM users WHERE email = $1", form.email)
        .fetch_one(&state.postgres)
        .await?;
    let password_result = state
        .spawn_rayon(move |state| {
            let hash = PasswordHash::new(&output.password)?;
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
    Ok((
        cookies.add(Cookie::new("auth", token)),
        Redirect::to(&state.config.root_url),
    ))
}
