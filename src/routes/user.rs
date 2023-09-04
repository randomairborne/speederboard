use crate::{model::User, template::BaseRenderInfo, AppState, Error, HandlerResult};
use axum::extract::{Path, State};

#[derive(serde::Serialize, Debug, Clone)]
pub struct UserPage {
    #[serde(flatten)]
    core: BaseRenderInfo,
    user: User,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    Path(username): Path<String>,
    core: BaseRenderInfo,
) -> HandlerResult {
    let user = query_as!(
        User,
        "SELECT
        id, username, has_stylesheet, pfp_ext, banner_ext,
        biography, admin, created_at
        FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let ctx = UserPage { core, user };
    state.render("user.jinja", ctx)
}
