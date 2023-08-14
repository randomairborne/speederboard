use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::rngs::OsRng;

use crate::{
    error::ArgonError,
    id::{Id, UserMarker},
    model::User,
};

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
