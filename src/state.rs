use crate::{
    config::Config,
    id::{Id, UserMarker},
    template::BaseRenderInfo,
    user::User,
    Error,
};
use argon2::Argon2;
use deadpool_redis::Pool as RedisPool;
use rayon::ThreadPool;
use redis::AsyncCommands;
use sqlx::PgPool;
use std::sync::Arc;
use tera::Tera;

pub type AppState = Arc<InnerAppState>;

#[derive(Clone)]
pub struct InnerAppState {
    pub config: Config,
    pub tera: Tera,
    pub redis: RedisPool,
    pub postgres: PgPool,
    pub rayon: Arc<ThreadPool>,
    pub argon: Argon2<'static>,
    pub http: reqwest::Client,
}

impl InnerAppState {
    /// # Errors
    /// If somehow the channel hangs up, this can error.
    pub async fn spawn_rayon<O, F>(
        &self,
        func: F,
    ) -> Result<O, tokio::sync::oneshot::error::RecvError>
    where
        O: Send + 'static,
        F: FnOnce(InnerAppState) -> O + Send + 'static,
    {
        let state = self.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.rayon.spawn(move || {
            let _ = tx.send(func(state));
        });
        rx.await
    }
    #[must_use]
    pub fn base_context(&self) -> BaseRenderInfo {
        BaseRenderInfo::new(&self.config.root_url, &self.config.cdn_url)
    }
    pub async fn update_user(&self, user: DbUserUpdate) -> Result<User, Error> {
        let new_db_user = query_as!(
            User,
            "UPDATE users SET
                username = COALESCE($2, username),
                has_stylesheet = COALESCE($3, has_stylesheet),
                biography = COALESCE($4, biography),
                pfp_ext = CASE WHEN $5 THEN NULL ELSE COALESCE($6, pfp_ext) END,
                banner_ext = CASE WHEN $7 THEN NULL ELSE COALESCE($8, banner_ext) END
            WHERE id = $1
            RETURNING id, username, has_stylesheet,
            pfp_ext, banner_ext, biography",
            user.id.get(),
            user.username,
            user.has_stylesheet,
            user.biography,
            user.pfp_ext.is_null(),
            user.pfp_ext.into_option(),
            user.banner_ext.is_null(),
            user.banner_ext.into_option(),
        )
        .fetch_one(&self.postgres)
        .await?;
        self.redis
            .get()
            .await?
            .set_ex(
                format!("user:{}", user.id.get()),
                serde_json::to_string(&new_db_user)?,
                86_400,
            )
            .await?;
        Ok(new_db_user)
    }
    pub async fn put_r2_file(
        &self,
        location: &str,
        file: reqwest::Body,
        content_type: &str,
    ) -> Result<(), Error> {
        let resp = self
            .http
            .put(format!("{}{}", self.config.fakes3_endpoint, location))
            .bearer_auth(&self.config.fakes3_token)
            .header("content-type", content_type)
            .body(file)
            .send()
            .await?
            .error_for_status()?;
        trace!(?resp, "got response on creation");
        Ok(())
    }
    pub async fn delete_r2_file(&self, location: &str) -> Result<(), Error> {
        let resp = self
            .http
            .delete(format!("{}{}", self.config.fakes3_endpoint, location))
            .bearer_auth(&self.config.fakes3_token)
            .send()
            .await?
            .error_for_status()?;
        trace!(?resp, "got response on deletion");
        Ok(())
    }
}

#[derive(Clone)]
pub struct DbUserUpdate {
    id: Id<UserMarker>,
    username: Option<String>,
    has_stylesheet: Option<bool>,
    biography: Option<String>,
    pfp_ext: MaybeNullUpdate<String>,
    banner_ext: MaybeNullUpdate<String>,
}

impl DbUserUpdate {
    pub fn new(id: Id<UserMarker>) -> Self {
        Self {
            id,
            username: None,
            has_stylesheet: None,
            biography: None,
            pfp_ext: MaybeNullUpdate::None,
            banner_ext: MaybeNullUpdate::None,
        }
    }
    pub fn username(self, username: String) -> Self {
        Self {
            username: Some(username),
            ..self
        }
    }
    pub fn has_stylesheet(self, has_stylesheet: bool) -> Self {
        Self {
            has_stylesheet: Some(has_stylesheet),
            ..self
        }
    }
    pub fn biography(self, biography: String) -> Self {
        Self {
            biography: Some(biography),
            ..self
        }
    }
    pub fn pfp_ext(self, pfp_ext: Option<String>) -> Self {
        Self {
            pfp_ext: pfp_ext.into(),
            ..self
        }
    }
    pub fn banner_ext(self, banner_ext: Option<String>) -> Self {
        Self {
            banner_ext: banner_ext.into(),
            ..self
        }
    }
}

#[derive(Clone)]
pub enum MaybeNullUpdate<T: Clone> {
    Null,
    None,
    Some(T),
}

impl<T: Clone> MaybeNullUpdate<T> {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    pub fn into_option(self) -> Option<T> {
        self.into()
    }
}

impl<T: Clone> From<Option<T>> for MaybeNullUpdate<T> {
    fn from(value: Option<T>) -> Self {
        if let Some(v) = value {
            Self::Some(v)
        } else {
            Self::Null
        }
    }
}

impl<T: Clone> From<MaybeNullUpdate<T>> for Option<T> {
    fn from(value: MaybeNullUpdate<T>) -> Option<T> {
        if let MaybeNullUpdate::Some(v) = value {
            Some(v)
        } else {
            None
        }
    }
}
