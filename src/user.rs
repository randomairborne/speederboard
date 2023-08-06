use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    id::{Id, UserMarker},
    AppState, Error,
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct User {
    pub id: Id<UserMarker>,
    pub username: String,
    pub has_stylesheet: bool,
    pub biography: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp_ext: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_ext: Option<String>,
}

const DEFAULT_PFP: &str = "/static/pfp/default.png";

impl User {
    pub fn banner_dest_path(&self) -> String {
        format!("/customfiles/users/{}/banner.png", self.id)
    }
    pub fn pfp_dest_path(&self) -> String {
        format!("/customfiles/users/{}/pfp.png", self.id)
    }
    pub fn stylesheet_dest_path(&self) -> String {
        format!("/customfiles/users/{}/style.css", self.id)
    }
    pub fn stylesheet(&self) -> Option<String> {
        if self.has_stylesheet {
            Some(self.stylesheet_dest_path())
        } else {
            None
        }
    }
    pub fn pfp_path(&self) -> String {
        if self.pfp_ext.is_some() {
            self.pfp_dest_path()
        } else {
            DEFAULT_PFP.to_string()
        }
    }
    pub fn banner_path(&self) -> Option<String> {
        if self.has_stylesheet {
            Some(self.banner_dest_path())
        } else {
            None
        }
    }
}

pub const TOKEN_COOKIE: &str = "token";

#[axum::async_trait]
impl FromRequestParts<AppState> for User {
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
