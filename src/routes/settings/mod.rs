pub mod credentials;
pub mod files;
use crate::{id::Id, state::DbUserUpdate, template::BaseRenderInfo, user::User, AppState, Error};
use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};

#[derive(serde::Serialize)]
pub struct SettingsUserContext<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
    user: PrivateUser,
}

#[derive(serde::Serialize)]
pub struct PrivateUser {
    #[serde(flatten)]
    base: User,
    email: String,
}

#[derive(serde::Deserialize)]
pub struct UserUpdate {
    pub username: String,
    pub biography: String,
}

const UPDATE_COMPLETE_URL: &str = "/settings?updated=true";

#[allow(clippy::unused_async)]
pub async fn get(State(state): State<AppState>, user: User) -> Result<Html<String>, Error> {
    let query = query!(
        "SELECT
        id, username, has_stylesheet, pfp_ext, banner_ext, biography, email
        FROM users WHERE id = $1",
        user.id.get()
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let base_user = User {
        id: Id::new(query.id),
        username: query.username,
        has_stylesheet: query.has_stylesheet,
        biography: query.biography,
        pfp_ext: query.pfp_ext,
        banner_ext: query.banner_ext,
    };
    let private_user = PrivateUser {
        base: base_user,
        email: query.email,
    };
    let mut core = state.base_context();
    core.logged_in_user = Some(user.clone());
    let ctx = SettingsUserContext {
        core,
        user: private_user,
    };
    let context_ser = tera::Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("settings.jinja", &context_ser)?))
}

pub async fn profile(
    State(state): State<AppState>,
    user: User,
    Form(form): Form<UserUpdate>,
) -> Result<Redirect, Error> {
    let update = DbUserUpdate::new(user.id)
        .username(form.username)
        .biography(form.biography);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}
