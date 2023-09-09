use std::fmt::Debug;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::extract::FromRequest;
use rand::rngs::OsRng;

use crate::{
    error::ArgonError,
    id::{Id, UserMarker},
    model::{Game, Member, Permissions, User},
};

pub const MIN_PASSWORD_LEN: usize = 8;

pub const MAX_EMAIL_LEN: usize = 255;
pub const MIN_EMAIL_LEN: usize = 5;
pub const MAX_USERNAME_LEN: usize = 16;
pub const MIN_USERNAME_LEN: usize = 2;
pub const MAX_USER_BIOGRAPHY_LEN: usize = 4000;
pub const MIN_USER_BIOGRAPHY_LEN: usize = 0;
pub const MAX_GAME_NAME_LEN: usize = 128;
pub const MIN_GAME_NAME_LEN: usize = 1;
pub const MAX_GAME_SLUG_LEN: usize = 32;
pub const MIN_GAME_SLUG_LEN: usize = 2;
pub const MAX_GAME_URL_LEN: usize = 128;
pub const MIN_GAME_URL_LEN: usize = 12;
pub const MAX_GAME_DESCRIPTION_LEN: usize = 4000;
pub const MIN_GAME_DESCRIPTION_LEN: usize = 0;
pub const MAX_CATEGORY_NAME_LEN: usize = 128;
pub const MIN_CATEGORY_NAME_LEN: usize = 2;
pub const MAX_CATEGORY_DESCRIPTION_LEN: usize = 4000;
pub const MIN_CATEGORY_DESCRIPTION_LEN: usize = 0;
pub const MAX_CATEGORY_RULES_LEN: usize = 20_000;
pub const MIN_CATEGORY_RULES_LEN: usize = 0;
pub const MAX_RUN_VIDEO_LEN: usize = 256;
pub const MIN_RUN_VIDEO_LEN: usize = 12;
pub const MAX_RUN_DESCRIPTION_LEN: usize = 4000;
pub const MIN_RUN_DESCRIPTION_LEN: usize = 0;
pub const MAX_FORUM_TITLE_LEN: usize = 128;
pub const MIN_FORUM_TITLE_LEN: usize = 5;
pub const MAX_FORUM_POST_LEN: usize = 4000;
pub const MIN_FORUM_POST_LEN: usize = 1;
pub const MAX_FORUM_COMMENT_LEN: usize = 4000;
pub const MIN_FORUM_COMMENT_LEN: usize = 1;

pub const AUTHTOKEN_COOKIE: &str = "token";
pub const AUTHTOKEN_TTL: usize = 24 * 60 * 60;

pub fn default_return_to() -> String {
    String::from('/')
}

pub fn return_false() -> bool {
    false
}

pub fn return_0_i64() -> i64 {
    0
}

pub fn return_0_usize() -> usize {
    0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn validate_slug(value: &str, _context: &()) -> garde::Result {
    if !value.chars().all(|char| {
        (char.is_ascii_alphanumeric() && char.is_ascii_lowercase())
            || char == '_'
            || char == '-'
            || char == '.'
    }) {
        return Err(garde::Error::new(
            "game slug must contain only lowercase alphanumeric characters, _, -, and .",
        ));
    }
    Ok(())
}

pub fn hash_password(password: &[u8], argon: &Argon2) -> Result<String, ArgonError> {
    trace!("hashing password");
    let salt = SaltString::generate(&mut OsRng);
    argon
        .hash_password(password, &salt)
        .map_err(Into::into)
        .map(|v| v.to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn opt_user(
    id: Option<Id<UserMarker>>,
    name: Option<String>,
    has_stylesheet: Option<bool>,
    bio: Option<String>,
    pfp_ext: Option<String>,
    banner_ext: Option<String>,
    admin: Option<bool>,
    created_at: Option<chrono::NaiveDateTime>,
    flags: Option<i64>,
) -> Option<User> {
    Some(User {
        id: id?,
        username: name?,
        has_stylesheet: has_stylesheet?,
        biography: bio?,
        pfp_ext,
        banner_ext,
        admin: admin?,
        created_at: created_at?,
        flags: flags?,
    })
}

pub struct ValidatedForm<T>(pub T);

#[axum::async_trait]
impl<S, B, T> FromRequest<S, B> for ValidatedForm<T>
where
    T: serde::de::DeserializeOwned + garde::Validate<Context = ()> + Debug,
    B: axum::body::HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<axum::BoxError>,
    S: Send + Sync,
{
    type Rejection = crate::Error;

    async fn from_request(req: axum::http::Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let form: T = axum::Form::from_request(req, state).await?.0;
        trace!(data = ?form, "deserialized form-data, validating");
        form.validate(&())?;
        Ok(Self(form))
    }
}

pub async fn game_n_member(
    state: &crate::AppState,
    user: User,
    game_slug: &str,
) -> Result<(Game, Member), crate::Error> {
    trace!(userid = ?user.id, username = user.username, game_slug, "fetching user permissions for game");
    let data = query!(
        "SELECT g.id, g.name, g.slug,
        g.url, g.default_category, g.description,
        g.has_stylesheet, g.banner_ext,
        g.cover_art_ext, g.flags, p.permissions
        FROM games as g LEFT JOIN permissions as p
        ON p.user_id = $1 AND p.game_id = g.id AND g.slug = $2",
        user.id.get(),
        game_slug
    )
    .fetch_one(&state.postgres)
    .await?;
    let perms = if user.admin {
        trace!(
            user.username = user.username,
            user.id = ?user.id,
            game.id = data.id,
            game.name = data.name,
            game.slug = data.slug,
            "user has admin, overriding member permissions"
        );
        Permissions::ADMINISTRATOR
    } else {
        Permissions::new_opt(data.permissions)
    };
    let member = Member { perms, user };
    let game = Game {
        id: data.id.into(),
        name: data.name,
        slug: data.slug,
        url: data.url,
        default_category: data.default_category.into(),
        description: data.description,
        has_stylesheet: data.has_stylesheet,
        banner_ext: data.banner_ext,
        cover_art_ext: data.cover_art_ext,
        flags: data.flags,
    };
    Ok((game, member))
}
