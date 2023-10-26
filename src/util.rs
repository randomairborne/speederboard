use std::fmt::Debug;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{
    extract::{FromRequest, State},
    http::Request,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use rand::rngs::OsRng;
use redis::AsyncCommands;

use crate::{
    error::ArgonError,
    id::{Id, UserMarker},
    model::{Game, Member, Permissions, User},
    AppState, Error,
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
pub const AUTHTOKEN_TTL: usize = 24 * 60 * 60 * 7;
pub const AUTHTOKEN_TTL_I64: i64 = 24 * 60 * 60 * 7;

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
            "slug must contain only lowercase alphanumeric characters, _, -, and .",
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

// TODO: move this to an associated function on User

pub fn opt_user(
    id: Option<Id<UserMarker>>,
    name: Option<String>,
    has_stylesheet: Option<bool>,
    bio: Option<String>,
    pfp: Option<bool>,
    banner: Option<bool>,
    admin: Option<bool>,
    created_at: Option<chrono::NaiveDateTime>,
    flags: Option<i64>,
    language: Option<String>,
) -> Option<User> {
    Some(User {
        id: id?,
        username: name?,
        has_stylesheet: has_stylesheet?,
        biography: bio?,
        pfp: pfp?,
        banner: banner?,
        admin: admin?,
        created_at: created_at?,
        flags: flags?,
        language,
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
        g.has_stylesheet, g.banner,
        g.cover_art, g.flags, p.permissions
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
        banner: data.banner,
        cover_art: data.cover_art,
        flags: data.flags,
    };
    Ok((game, member))
}

// TODO: Move these two functions to methods of state

pub async fn get_redis_object<
    T: for<'de> serde::Deserialize<'de>,
    K: redis::ToRedisArgs + Send + Sync,
>(
    state: &AppState,
    key: K,
) -> Result<Option<T>, Error> {
    let maybe_object_str: Option<String> = state.redis.get().await?.get(key).await?;
    if let Some(object_str) = maybe_object_str {
        let object: T = serde_json::from_str(&object_str)?;
        Ok(Some(object))
    } else {
        Ok(None)
    }
}

pub async fn set_redis_object<K: redis::ToRedisArgs + Send + Sync, V: serde::Serialize>(
    state: &AppState,
    key: K,
    data: &V,
    expiry: usize,
) -> Result<(), Error> {
    let game_str = serde_json::to_string(data)?;
    state
        .redis
        .get()
        .await?
        .set_ex(key, game_str, expiry)
        .await?;
    Ok(())
}

pub async fn csp_middleware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let mut resp = next.run(request).await;
    resp.headers_mut()
        .insert("content-security-policy", state.csp());
    resp
}

pub fn auth_cookie<'a>(token: String) -> Cookie<'a> {
    Cookie::build(AUTHTOKEN_COOKIE, token)
        .secure(true)
        .http_only(true)
        .max_age(s3::creds::time::Duration::seconds(AUTHTOKEN_TTL_I64))
        .same_site(SameSite::Strict)
        .finish()
}
