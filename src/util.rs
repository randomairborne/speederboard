use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::rngs::OsRng;

use crate::error::ArgonError;

pub fn hash_password(password: &[u8], argon: &Argon2) -> Result<String, ArgonError> {
    let salt = SaltString::generate(&mut OsRng);
    argon
        .hash_password(password, &salt)
        .map_err(Into::into)
        .map(|v| v.to_string())
}

pub const AUTHTOKEN_COOKIE: &str = "token";
pub const AUTHTOKEN_TTL: usize = 24 * 60 * 60;
