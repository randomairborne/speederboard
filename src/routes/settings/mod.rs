pub mod credentials;
pub mod files;
use crate::{id::Id, model::User, state::DbUserUpdate, template::BaseRenderInfo, AppState, Error};
use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    Form,
};

#[derive(serde::Serialize)]
pub struct SettingsUserContext<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
    user: PrivateUser,
    updated: String,
}

#[derive(serde::Serialize)]
pub struct PrivateUser {
    #[serde(flatten)]
    base: User,
    email: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateQuery {
    #[serde(default = "empty_string")]
    pub updated: String,
}

fn empty_string() -> String {
    String::new()
}

#[derive(serde::Deserialize)]
pub struct UserUpdate {
    pub username: String,
    pub biography: String,
}

const UPDATE_COMPLETE_URL: &str = "/settings?updated=true";

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    user: User,
    Query(query): Query<UpdateQuery>,
) -> Result<Html<String>, Error> {
    let record = query!(
        "SELECT
        id, username, has_stylesheet, pfp_ext, banner_ext, biography, email, admin
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
    };
    let private_user = PrivateUser {
        base: base_user,
        email: record.email,
    };
    let mut core = state.base_context();
    core.logged_in_user = Some(user.clone());
    let ctx = SettingsUserContext {
        core,
        user: private_user,
        updated: query.updated,
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
