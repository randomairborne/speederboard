use std::{fmt::Display, sync::OnceLock};

use crate::AppState;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
pub static ERROR_STATE: OnceLock<AppState> = OnceLock::new();

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
    #[error("JSON serialization or deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("s3 library error: {0}")]
    S3(#[from] s3::error::S3Error),
    #[error("multipart upload error: {0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error("format error: {0}")]
    InvalidMultipart(&'static str),
    #[error("This should be impossible. {0}")]
    Impossible(#[from] std::convert::Infallible),
    #[error("Field {0} must {1}")]
    FormValidation(&'static str, &'static str),
    #[error("Username or password is incorrect")]
    InvalidPassword,
    #[error("Invalid auth cookie")]
    InvalidCookie,
    #[error(
        "This token has a valid ID associated with it, but no data is associated with its ID."
    )]
    TokenHasIdButIdIsUnkown,
    #[error("Not found")]
    NotFound,
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let state = ERROR_STATE.get().unwrap();
        let status = match &self {
            Self::Sqlx(_)
            | Self::DeadpoolRedis(_)
            | Self::Redis(_)
            | Self::Tera(_)
            | Self::Argon2(_)
            | Self::OneshotRecv(_)
            | Self::SerdeJson(_)
            | Self::S3(_)
            | Self::Impossible(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FormValidation(_, _)
            | Self::Multipart(_)
            | Self::InvalidMultipart(_)
            | Self::TokenHasIdButIdIsUnkown => StatusCode::BAD_REQUEST,
            Self::InvalidPassword => StatusCode::FORBIDDEN,
            Self::InvalidCookie => return Redirect::to("/login").into_response(),
            Self::NotFound => return crate::routes::notfound(state).into_response(),
        };
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            error!(?self, "failed to handle request");
        }
        let self_as_string = self.to_string();
        let mut ctx = match tera::Context::from_serialize(state.base_context()) {
            Ok(v) => v,
            Err(source) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format_raw_error(&self_as_string, &source.to_string()),
                )
                    .into_response()
            }
        };
        ctx.insert("error", &self_as_string);
        let content = state
            .tera
            .render("error.jinja", &ctx)
            .map(Html)
            .map_err(|source| {
                error!(?source, "failed to render error");
                format_raw_error(&self_as_string, &source.to_string())
            });
        (status, content).into_response()
    }
}

fn format_raw_error(original: &str, tera: &str) -> String {
    format!(
        "There was an error handling your request.
In addition, there was an error attempting to use tera to template said error.
original error: `{original}`
tera error: `{tera}`
Please send an email to valk@randomairborne.dev with a copy of this message."
    )
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
