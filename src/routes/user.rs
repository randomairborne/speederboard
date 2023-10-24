use axum::extract::{Path, State};

use crate::{model::User, template::BaseRenderInfo, AppState, Error, HandlerResult};

#[derive(serde::Serialize, Debug, Clone)]
pub struct UserPage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    user: User,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    Path(username): Path<String>,
    base: BaseRenderInfo,
) -> HandlerResult {
    let user = query_as!(
        User,
        "SELECT
        id, username, has_stylesheet, pfp, banner,
        biography, admin, created_at, flags, language
        FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let ctx = UserPage { base, user };
    state.render("user.jinja", ctx)
}
