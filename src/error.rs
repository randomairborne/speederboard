use std::fmt::Display;

use axum::{
    extract::{OriginalUri, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{template::BaseRenderInfo, AppState};

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
    MultiFormValidation(#[from] garde::Report),
    #[error("Form data invalid: {0}")]
    FormRejection(#[from] axum::extract::rejection::FormRejection),
    #[error("Failed to validate submission: {0}")]
    CustomFormValidation(String),
    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Failed to parse or render image: {0}")]
    Image(#[from] image::ImageError),
    #[error("URL missing query pair {0}")]
    MissingQueryPair(&'static str),
    #[error("This endpoint needs authorization")]
    NeedsLogin(String),
    #[error("Query expected to return {0} rows returned {1}")]
    TooManyRows(usize, usize),
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
    #[error("This post is a root post (it has no parent) but it has no title!")]
    NoTitleForRootPost,
    #[error(
        "You can't delete the default category for a game, change the default category first!!"
    )]
    CannotDeleteDefaultCategory,
    #[error("URL being parsed does not have a domain!")]
    NoDomainInUrl,
    #[error("URL being parsed has `..`, which could indicate a directory traversal attack!")]
    DoubleDotInPath,
    #[error("Translation file did not have stem!")]
    NoFileStem,
    #[error("Invalid translation filename OS string!")]
    InvalidOsString,
    #[error("Hit root directory while parsing path!")]
    PathHasNoParent,
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
    fn into_response(self) -> Response {
        let mut resp = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response();
        trace!(?self, "Converting error into response");
        resp.extensions_mut().insert(self);
        resp
    }
}
pub async fn error_middleware<B>(
    State(state): State<AppState>,
    uri: OriginalUri,
    base: BaseRenderInfo,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let response = next.run(request).await;
    let error: &Error = if let Some(v) = response.extensions().get() {
        v
    } else {
        return response;
    };
    let status = match &error {
        Error::Sqlx(_)
        | Error::DeadpoolRedis(_)
        | Error::Redis(_)
        | Error::Tera(_)
        | Error::Argon2(_)
        | Error::OneshotRecv(_)
        | Error::SerdeJson(_)
        | Error::Reqwest(_)
        | Error::Impossible(_)
        | Error::TaskJoin(_)
        | Error::Io(_)
        | Error::Format(_)
        | Error::TryFromInt(_)
        | Error::UrlParse(_)
        | Error::MissingQueryPair(_)
        | Error::TooManyRows(_, _)
        | Error::NoTitleForRootPost
        | Error::RowDoesNotMatchInputGame
        | Error::NoDomainInUrl
        | Error::PathHasNoParent
        | Error::NoFileStem
        | Error::InvalidOsString => StatusCode::INTERNAL_SERVER_ERROR,
        Error::FormValidation(_)
        | Error::CustomFormValidation(_)
        | Error::FormRejection(_)
        | Error::MultiFormValidation(_)
        | Error::Multipart(_)
        | Error::InvalidMultipart(_)
        | Error::Image(_)
        | Error::TokenHasIdButIdIsUnkown
        | Error::InvalidGameCategoryPair
        | Error::CannotDeleteDefaultCategory
        | Error::DoubleDotInPath => StatusCode::BAD_REQUEST,
        Error::InvalidPassword | Error::InsufficientPermissions => StatusCode::UNAUTHORIZED,
        Error::InvalidCookie => return state.redirect("/login").into_response(),
        Error::NeedsLogin(return_to) => {
            return state
                .redirect(format!("/login?return_to={return_to}"))
                .into_response()
        }
        Error::NotFound => {
            return crate::routes::notfound(&state, base, uri.to_string()).into_response()
        }
    };
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        error!(?error, "failed to handle request");
    }
    let error_as_string = error.to_string();
    let mut ctx = match tera::Context::from_serialize(base) {
        Ok(v) => v,
        Err(source) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format_raw_error(&error_as_string, &source.to_string()),
            )
                .into_response()
        }
    };
    ctx.insert("error", &error_as_string);
    let content = state.render_ctx("error.jinja", &ctx).map_err(|source| {
        error!(?source, "failed to render error");
        format_raw_error(&error_as_string, &source.to_string())
    });
    (status, [("cache-control", "private")], content).into_response()
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
