use axum::extract::{Path, State};

use crate::{id::Id, model::User, template::BaseRenderInfo, AppState, Error, HandlerResult};

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
    let row = query!(
        "SELECT
        id, username, stylesheet, pfp, banner,
        biography, admin, created_at, flags, language
        FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let user = User {
        id: Id::new(row.id),
        username: row.username,
        stylesheet: row.stylesheet,
        biography: row.biography,
        pfp: row.pfp,
        banner: row.banner,
        admin: row.admin,
        created_at: row.created_at,
        flags: row.flags,
        language: None,
    };
    let ctx = UserPage { base, user };
    state.render("user.jinja", ctx)
}
