use crate::{
    template::BaseRenderInfo,
    user::{User, TOKEN_COOKIE},
    AppState, Error,
};
use argon2::{PasswordHash, PasswordVerifier};
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use rand::distributions::DistString;
use tera::Context;

#[derive(serde::Serialize)]
pub struct UserPage<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
    user: User,
}

#[derive(serde::Deserialize)]
pub struct UserUpdateForm {
    username: Option<String>,
    biography: Option<String>,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    Path(username): Path<String>,
    logged_in_user: Option<User>,
) -> Result<Html<String>, Error> {
    let user = query_as!(
        User,
        "SELECT
        id, username, has_stylesheet, pfp_ext, banner_ext, biography
        FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let mut core = state.base_context();
    core.logged_in_user = logged_in_user;
    let ctx = UserPage { core, user };
    let context_ser = Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("user.jinja", &context_ser)?))
}

pub async fn post(
    State(state): State<AppState>,
    user: User,
    Form(form): Form<UserUpdateForm>,
) -> Result<Redirect, Error> {
    let own_page = format!("/user/{}", user.username);
    let Some(output) = query!(
        "UPDATE users
        SET username = $1
        WHERE id = $1",
        user.id.get()
    )
    .fetch_optional(&state.postgres)
    .await? else {
        return Ok(Redirect::to(&own_page));
    };
    Ok(Redirect::to(&own_page))
}
