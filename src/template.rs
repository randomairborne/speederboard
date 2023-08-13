use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{model::User, AppState};

#[derive(serde::Serialize)]
pub struct BaseRenderInfo {
    pub root_url: String,
    pub cdn_url: String,
    pub logged_in_user: Option<User>,
}

impl BaseRenderInfo {
    pub fn new(root_url: String, cdn_url: String) -> Self {
        Self {
            root_url,
            cdn_url,
            logged_in_user: None,
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for BaseRenderInfo {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = User::from_request_parts(parts, state).await.ok();
        let bri = BaseRenderInfo {
            root_url: state.config.root_url.clone(),
            cdn_url: state.config.cdn_url.clone(),
            logged_in_user: user,
        };
        Ok(bri)
    }
}
