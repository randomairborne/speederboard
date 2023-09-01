pub mod credentials;
pub mod files;
use crate::{
    id::Id, model::User, state::DbUserUpdate, template::BaseRenderInfo, util::ValidatedForm,
    AppState, Error,
};
use axum::{
    extract::State,
    response::{Html, Redirect},
};

#[derive(serde::Serialize)]
pub struct SettingsUserContext {
    #[serde(flatten)]
    base: BaseRenderInfo,
    user: PrivateUser,
}

#[derive(serde::Serialize)]
pub struct PrivateUser {
    #[serde(flatten)]
    base: User,
    email: String,
}

#[derive(serde::Deserialize, garde::Validate)]
pub struct UserUpdate {
    #[garde(length(min = crate::util::MIN_USERNAME_LEN, max = crate::util::MAX_USERNAME_LEN))]
    pub username: String,
    #[garde(length(min = crate::util::MIN_USER_BIOGRAPHY_LEN, max = crate::util::MAX_USER_BIOGRAPHY_LEN))]
    pub biography: String,
}

const UPDATE_COMPLETE_URL: &str = "/settings";

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    user: User,
    base: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    let record = query!(
        "SELECT
        id, username, has_stylesheet, pfp_ext, banner_ext,
        biography, email, admin, created_at
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
    };
    let private_user = PrivateUser {
        base: base_user,
        email: record.email,
    };
    let ctx = SettingsUserContext {
        base,
        user: private_user,
    };
    let context_ser = tera::Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("settings.jinja", &context_ser)?))
}

pub async fn profile(
    State(state): State<AppState>,
    user: User,
    ValidatedForm(form): ValidatedForm<UserUpdate>,
) -> Result<Redirect, Error> {
    let update = DbUserUpdate::new(user.id)
        .username(form.username)
        .biography(form.biography);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}
