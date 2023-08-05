use std::{fmt::Display, sync::OnceLock};

use axum::{http::StatusCode, response::IntoResponse};
use tera::{Context, Tera};

pub static ERROR_TERA: OnceLock<Tera> = OnceLock::new();

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Redis Deadpool error: {0}")]
    DeadpoolRedis(#[from] deadpool_redis::PoolError),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Tera error: {0}")]
    Tera(#[from] tera::Error),
    #[error("Argon2 error")]
    Argon2(#[from] ArgonError),
    #[error("Oneshot channel recv error: {0}")]
    OneshotRecv(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("Field {0} must {1}")]
    FormValidation(&'static str, &'static str),
    #[error("Username or password is incorrect")]
    InvalidPassword,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::Sqlx(_)
            | Self::DeadpoolRedis(_)
            | Self::Redis(_)
            | Self::Tera(_)
            | Self::Argon2(_)
            | Self::OneshotRecv(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FormValidation(_, _) => StatusCode::BAD_REQUEST,
            Self::InvalidPassword => StatusCode::FORBIDDEN,
        };
        let tera = ERROR_TERA.get().unwrap();
        let self_as_string = self.to_string();
        let mut ctx = Context::new();
        ctx.insert("error", &self_as_string);
        let content = tera.render("error.jinja", &ctx).map_err(|e| {
            format!(
                "
There was an error handling your request.
In addition, there was an error attempting to use tera to template your request.
tera error: `{e}`
original error: `{self_as_string}`
Please send an email to valk@randomairborne.dev with a copy of this message."
            )
        });
        (status, content).into_response()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ArgonError {
    PasswordHash(argon2::password_hash::Error),
    Argon2(argon2::Error),
}

impl std::error::Error for ArgonError {}

impl Display for ArgonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<argon2::password_hash::Error> for ArgonError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(value)
    }
}

impl From<argon2::Error> for ArgonError {
    fn from(value: argon2::Error) -> Self {
        Self::Argon2(value)
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Argon2(ArgonError::PasswordHash(value))
    }
}

impl From<argon2::Error> for Error {
    fn from(value: argon2::Error) -> Self {
        Self::Argon2(ArgonError::Argon2(value))
    }
}
