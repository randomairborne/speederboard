pub mod credentials;
pub mod files;

use axum::{
    extract::{Query, State},
    response::Redirect,
};
use strum::IntoEnumIterator;

use crate::{
    id::Id,
    language::Language,
    model::{User, UserUpdate},
    template::BaseRenderInfo,
    util::ValidatedForm,
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct SettingsPage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    user: PrivateUser,
    incorrect: bool,
    languages: Vec<LanguageMetadata>,
    custom_styles_disabled: bool,
    js_url: String,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct LanguageMetadata {
    code: &'static str,
    name: &'static str,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SettingsQuery {
    #[serde(default = "crate::util::return_false")]
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
    #[garde(length(min = crate::util::MIN_USERNAME_LEN, max = crate::util::MAX_USERNAME_LEN), custom(crate::util::validate_slug))]
    username: String,
    #[garde(length(min = crate::util::MIN_USER_BIOGRAPHY_LEN, max = crate::util::MAX_USER_BIOGRAPHY_LEN))]
    biography: String,
    #[garde(skip)]
    #[serde(deserialize_with = "language_option_sentinel")]
    language: Option<Language>,
}

fn language_option_sentinel<'de, D>(input: D) -> Result<Option<Language>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let data: String = serde::Deserialize::deserialize(input)?;
    if data == "unset" {
        return Ok(None);
    }
    let data = Language::from_lang_code(&data)
        .ok_or(serde::de::Error::unknown_field(&data, &Language::CODES))?;
    Ok(Some(data))
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
        id, username, stylesheet, pfp, banner,
        biography, email, admin, created_at, flags, language
        FROM users WHERE id = $1",
        user.id.get()
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let base_user = User {
        id: Id::new(record.id),
        username: record.username,
        stylesheet: record.stylesheet,
        biography: record.biography,
        pfp: record.pfp,
        banner: record.banner,
        admin: record.admin,
        created_at: record.created_at,
        flags: record.flags,
        language: record.language,
    };
    let private_user = PrivateUser {
        base: base_user,
        email: record.email,
    };
    let ctx = SettingsPage {
        base,
        incorrect: query.incorrect,
        user: private_user,
        languages: Language::iter()
            .map(|lang| LanguageMetadata {
                code: lang.lang_code(),
                name: lang.display(),
            })
            .collect(),
        custom_styles_disabled: true,
        js_url: state.static_resource("/page-scripts/settings.js"),
    };
    state.render("settings.jinja", ctx)
}

pub async fn profile(
    State(state): State<AppState>,
    user: User,
    ValidatedForm(form): ValidatedForm<UserUpdateForm>,
) -> Result<Redirect, Error> {
    let update = UserUpdate::new(user.id)
        .language(form.language)
        .username(form.username)
        .biography(form.biography);
    update.execute(&state).await?;
    Ok(state.redirect("/settings"))
}
