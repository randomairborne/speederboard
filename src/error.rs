use std::{fmt::Display, sync::OnceLock};

use crate::{template::BaseRenderInfo, AppState};
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
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("multipart upload error: {0}")]
    Multipart(#[from] axum_extra::extract::multipart::MultipartError),
    #[error("dependency tokio task panicked: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
    #[error("integer out of range (this is a bug): {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
    #[error("format error: {0}")]
    InvalidMultipart(&'static str),
    #[error("This should be impossible. {0}")]
    Impossible(#[from] std::convert::Infallible),
    #[error("Formatting error: {0}")]
    Format(#[from] std::fmt::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to validate submission: {0}")]
    FormValidation(#[from] garde::Error),
    #[error("Failed to validate submission: \n{0}")]
    MultiFormValidation(#[from] garde::Errors),
    #[error("Form data invalid: {0}")]
    FormRejection(#[from] axum::extract::rejection::FormRejection),
    #[error("Failed to validate submission: {0}")]
    CustomFormValidation(String),
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
    #[error("This resource exists, but you do not have permission to access it")]
    InsufficientPermissions,
    #[error("That category isn't part of that game!")]
    InvalidGameCategoryPair,
    #[error(
        "You can't delete the default category for a game, change the default category first!!"
    )]
    CannotDeleteDefaultCategory,
    #[error(
        "in src/model/run.rs, in method ResolvedRun::row_to_rcat, \
        the game passed to the function does not match the parent game \
        ID of the record returned from the database. This is a bug."
    )]
    RowDoesNotMatchInputGame,
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let state = ERROR_STATE.get().unwrap();
        let core = BaseRenderInfo::new(state.config.root_url.clone(), state.config.cdn_url.clone());
        let status = match &self {
            Self::Sqlx(_)
            | Self::DeadpoolRedis(_)
            | Self::Redis(_)
            | Self::Tera(_)
            | Self::Argon2(_)
            | Self::OneshotRecv(_)
            | Self::SerdeJson(_)
            | Self::Reqwest(_)
            | Self::Impossible(_)
            | Self::TaskJoin(_)
            | Self::Io(_)
            | Self::Format(_)
            | Self::TryFromInt(_)
            | Self::RowDoesNotMatchInputGame => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FormValidation(_)
            | Self::CustomFormValidation(_)
            | Self::FormRejection(_)
            | Self::MultiFormValidation(_)
            | Self::Multipart(_)
            | Self::InvalidMultipart(_)
            | Self::TokenHasIdButIdIsUnkown
            | Self::InvalidGameCategoryPair
            | Self::CannotDeleteDefaultCategory => StatusCode::BAD_REQUEST,
            Self::InvalidPassword | Self::InsufficientPermissions => StatusCode::UNAUTHORIZED,
            Self::InvalidCookie => return Redirect::to("/login").into_response(),
            Self::NotFound => return crate::routes::notfound(state, core).into_response(),
        };
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            error!(?self, "failed to handle request");
        }
        let self_as_string = self.to_string();
        let mut ctx = match tera::Context::from_serialize(core) {
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
