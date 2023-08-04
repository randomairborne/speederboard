use std::sync::OnceLock;

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
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
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
        (StatusCode::INTERNAL_SERVER_ERROR, content).into_response()
    }
}
