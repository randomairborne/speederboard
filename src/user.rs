use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    id::{Id, UserMarker},
    AppState, Error,
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AuthUser {
    pub id: Id<UserMarker>,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FrontendUser {
    pub id: Id<UserMarker>,
    pub username: String,
    pub has_stylesheet: bool,
    pub pfp_ext: Option<String>,
    pub banner_ext: Option<String>,
}

pub const TOKEN_COOKIE: &str = "token";

#[axum::async_trait]
impl FromRequestParts<AppState> for FrontendUser {
    type Rejection = Error;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // this unwrap is safe because CookieJar's FromRequestParts is Infallible
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let cookie = jar.get(TOKEN_COOKIE).ok_or(Error::InvalidCookie)?;
        let maybe_user: Option<String> = state.redis.get().await?.get(cookie.value()).await?;
        let user = maybe_user.ok_or(Error::InvalidCookie)?;
        Ok(serde_json::from_str(&user)?)
    }
}
