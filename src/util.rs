use std::fmt::Debug;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{
    extract::{FromRequest, Request, State},
    http::{
        header::{CACHE_CONTROL, CONTENT_SECURITY_POLICY},
        HeaderValue,
    },
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use rand::rngs::OsRng;
use s3::creds::time::Duration as S3Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    error::ArgonError,
    model::{Game, Member, Permissions, User},
    AppState,
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

pub const MAX_PFP: ImageSizeLimit = ImageSizeLimit {
    width: 512,
    height: 512,
};

pub const MAX_BANNER: ImageSizeLimit = ImageSizeLimit {
    width: 2048,
    height: 1024,
};

pub const MAX_CSS_LEN: usize = 1024 * 50;

pub const AUTHTOKEN_COOKIE: &str = "token";
pub const AUTHTOKEN_TTL: u64 = 24 * 60 * 60 * 7;
pub const AUTHTOKEN_TTL_I64: i64 = 24 * 60 * 60 * 7;

static CACHE_CONTROL_VALUE: HeaderValue =
    HeaderValue::from_static("public,max-age=31536000,immutable");

#[derive(Debug, Copy, Clone)]
pub struct ImageSizeLimit {
    pub width: u32,
    pub height: u32,
}

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

pub struct ValidatedForm<T>(pub T);

#[axum::async_trait]
impl<S, T> FromRequest<S> for ValidatedForm<T>
where
    T: serde::de::DeserializeOwned + garde::Validate<Context = ()> + Debug,
    S: Send + Sync,
{
    type Rejection = crate::Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let form: T = axum::Form::from_request(req, state).await?.0;
        trace!(data = ?form, "deserialized form-data, validating");
        form.validate()?;
        Ok(Self(form))
    }
}

pub async fn game_n_member(
    state: &AppState,
    user: User,
    game_slug: &str,
) -> Result<(Game, Member), crate::Error> {
    trace!(userid = ?user.id, username = user.username, game_slug, "fetching user permissions for game");
    let data = query!(
        "SELECT g.id, g.name, g.slug,
        g.url, g.default_category, g.description,
        g.banner, g.cover_art, g.flags, p.permissions
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
        banner: data.banner,
        cover_art: data.cover_art,
        flags: data.flags,
    };
    Ok((game, member))
}

pub async fn csp_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let mut resp = next.run(request).await;
    resp.headers_mut()
        .insert(CONTENT_SECURITY_POLICY, state.csp());
    resp
}

pub async fn infinicache_middleware(request: Request, next: Next) -> Response {
    let mut resp = next.run(request).await;
    if resp.status().is_success() {
        resp.headers_mut()
            .insert(CACHE_CONTROL, CACHE_CONTROL_VALUE.clone());
    }
    resp
}

pub fn auth_cookie<'a>(token: String) -> Cookie<'a> {
    const AUTHTOKEN_TTL: S3Duration = S3Duration::seconds(AUTHTOKEN_TTL_I64);
    Cookie::build((AUTHTOKEN_COOKIE, token))
        .secure(true)
        .http_only(true)
        .max_age(AUTHTOKEN_TTL)
        .same_site(SameSite::Strict)
        .build()
}

pub fn start_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(concat!(env!("CARGO_PKG_NAME"), "=info").parse().unwrap())
        .with_env_var("LOG")
        .from_env()
        .expect("failed to parse env");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .init();
}
