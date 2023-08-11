use crate::{model::User, template::BaseRenderInfo, AppState, Error};
use axum::{
    extract::{Path, State},
    response::Html,
};
use tera::Context;

#[derive(serde::Serialize)]
pub struct UserPage<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
    user: User,
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
        id, username, has_stylesheet, pfp_ext, banner_ext, biography, admin
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
