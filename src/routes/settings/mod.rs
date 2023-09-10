pub mod credentials;
pub mod files;
use crate::{
    id::Id,
    model::{User, UserUpdate},
    template::BaseRenderInfo,
    util::ValidatedForm,
    AppState, Error, HandlerResult,
};
use axum::{
    extract::{Query, State},
    response::Redirect,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct SettingsPage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    user: PrivateUser,
    incorrect: bool,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SettingsQuery {
    incorrect: bool,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct PrivateUser {
    #[serde(flatten)]
    base: User,
    email: String,
}

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct UserUpdateForm {
    #[garde(length(min = crate::util::MIN_USERNAME_LEN, max = crate::util::MAX_USERNAME_LEN))]
    username: String,
    #[garde(length(min = crate::util::MIN_USER_BIOGRAPHY_LEN, max = crate::util::MAX_USER_BIOGRAPHY_LEN))]
    biography: String,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    user: User,
    base: BaseRenderInfo,
    Query(query): Query<SettingsQuery>,
) -> HandlerResult {
    let record = query!(
        "SELECT
        id, username, has_stylesheet, pfp_ext, banner_ext,
        biography, email, admin, created_at, flags
        FROM users WHERE id = $1",
        user.id.get()
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let base_user = User {
        id: Id::new(record.id),
        username: record.username,
        has_stylesheet: record.has_stylesheet,
        biography: record.biography,
        pfp_ext: record.pfp_ext,
        banner_ext: record.banner_ext,
        admin: record.admin,
        created_at: record.created_at,
        flags: record.flags,
    };
    let private_user = PrivateUser {
        base: base_user,
        email: record.email,
    };
    let ctx = SettingsPage {
        base,
        incorrect: query.incorrect,
        user: private_user,
    };
    state.render("settings.jinja", ctx)
}

pub async fn profile(
    State(state): State<AppState>,
    user: User,
    ValidatedForm(form): ValidatedForm<UserUpdateForm>,
) -> Result<Redirect, Error> {
    let update = UserUpdate::new(user.id)
        .username(form.username)
        .biography(form.biography);
    update.execute(&state).await?;
    Ok(state.redirect("location"))
}
