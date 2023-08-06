use crate::{template::BaseRenderInfo, user::User, AppState, Error};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Redirect},
    Form, Json,
};
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

#[derive(serde::Serialize)]
pub struct PresignEndpoint {
    put: String,
    del: String,
}

#[derive(serde::Serialize)]
pub struct PresignsResponse {
    stylesheet: PresignEndpoint,
    pfp: PresignEndpoint,
    banner: PresignEndpoint,
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

pub async fn put(
    State(state): State<AppState>,
    user: User,
    Form(form): Form<UserUpdateForm>,
) -> Result<Redirect, Error> {
    query!(
        "UPDATE users
        SET username = COALESCE($2, username),
        biography = COALESCE($3, username)
        WHERE id = $1",
        user.id.get(),
        form.username,
        form.biography
    )
    .execute(&state.postgres)
    .await?;
    Ok(Redirect::to("?updated=true"))
}

fn gen_presigns(state: &AppState, path: &str) -> Result<PresignEndpoint, s3::error::S3Error> {
    Ok(PresignEndpoint {
        put: state.s3.presign_put(path, 60, None)?,
        del: state.s3.presign_delete(path, 60)?,
    })
}

#[allow(clippy::unused_async)]
pub async fn presigns(
    State(state): State<AppState>,
    user: User,
) -> Result<Json<PresignsResponse>, Error<true>> {
    Ok(Json(PresignsResponse {
        stylesheet: gen_presigns(&state, &user.stylesheet_dest_path())?,
        pfp: gen_presigns(&state, &user.pfp_dest_path())?,
        banner: gen_presigns(&state, &user.banner_dest_path())?,
    }))
}

#[allow(clippy::unused_async)]
pub async fn extensions(
    State(state): State<AppState>,
    user: User,
) -> Result<StatusCode, Error<true>> {
    let banner_ext = if state.s3.head_object(user.banner_dest_path()).await?.1 == 404 {
        None
    } else {
        Some("png")
    };
    let pfp_ext = if state.s3.head_object(user.pfp_dest_path()).await?.1 == 404 {
        None
    } else {
        Some("png")
    };
    let has_stylesheet = state.s3.head_object(user.pfp_dest_path()).await?.1 == 404;
    query!(
        "UPDATE users SET
        banner_ext = $2,
        pfp_ext = $3,
        has_stylesheet = $4
        WHERE id = $1
     ",
        user.id.get(),
        banner_ext,
        pfp_ext,
        has_stylesheet
    )
    .execute(&state.postgres)
    .await?;
    Ok(StatusCode::ACCEPTED)
}
