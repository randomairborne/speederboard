use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::extract::FromRequest;
use rand::rngs::OsRng;

use crate::{
    error::ArgonError,
    id::{Id, UserMarker},
    model::User,
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
pub const MIN_CATEGORY_DESCRIPTION_LEN: usize = 100;
pub const MAX_CATEGORY_RULES_LEN: usize = 20_000;
pub const MIN_CATEGORY_RULES_LEN: usize = 100;
pub const MAX_RUN_VIDEO_LEN: usize = 256;
pub const MIN_RUN_VIDEO_LEN: usize = 12;
pub const MAX_RUN_DESCRIPTION_LEN: usize = 4000;
pub const MIN_RUN_DESCRIPTION_LEN: usize = 0;

pub const AUTHTOKEN_COOKIE: &str = "token";
pub const AUTHTOKEN_TTL: usize = 24 * 60 * 60;

pub fn hash_password(password: &[u8], argon: &Argon2) -> Result<String, ArgonError> {
    let salt = SaltString::generate(&mut OsRng);
    argon
        .hash_password(password, &salt)
        .map_err(Into::into)
        .map(|v| v.to_string())
}

pub fn opt_user(
    id: Option<Id<UserMarker>>,
    name: Option<String>,
    has_stylesheet: Option<bool>,
    bio: Option<String>,
    pfp_ext: Option<String>,
    banner_ext: Option<String>,
    admin: Option<bool>,
) -> Option<User> {
    Some(User {
        id: id?,
        username: name?,
        has_stylesheet: has_stylesheet?,
        biography: bio?,
        pfp_ext,
        banner_ext,
        admin: admin?,
    })
}

pub struct ValidatedForm<T>(pub T);

#[axum::async_trait]
impl<S, B, T> FromRequest<S, B> for ValidatedForm<T>
where
    T: serde::de::DeserializeOwned + garde::Validate<Context = ()>,
    B: axum::body::HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<axum::BoxError>,
    S: Send + Sync,
{
    type Rejection = crate::Error;
    async fn from_request(req: axum::http::Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let req: T = axum::Form::from_request(req, state).await?.0;
        req.validate(&())?;
        Ok(Self(req))
    }
}
