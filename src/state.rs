use crate::{
    config::Config,
    id::{Id, UserMarker},
    model::User,
    Error,
};
use argon2::Argon2;
use deadpool_redis::{Manager, Pool as RedisPool, Runtime};
use rayon::{ThreadPool, ThreadPoolBuilder};
use redis::AsyncCommands;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{sync::Arc, time::Duration};
use tera::Tera;

pub type AppState = Arc<InnerAppState>;

#[cfg(feature = "dev")]
pub type InnerTera = Arc<std::sync::RwLock<Tera>>;

#[cfg(not(feature = "dev"))]
pub type InnerTera = Tera;

#[derive(Clone)]
pub struct InnerAppState {
    pub config: Config,
    tera: InnerTera,
    pub redis: RedisPool,
    pub postgres: PgPool,
    rayon: Arc<ThreadPool>,
    pub argon: Argon2<'static>,
    pub http: reqwest::Client,
}

impl InnerAppState {
    pub fn new(
        config: Config,
        tera: InnerTera,
        redis: RedisPool,
        postgres: PgPool,
        rayon: Arc<ThreadPool>,
        argon: Argon2<'static>,
        http: reqwest::Client,
    ) -> Self {
        Self {
            config,
            tera,
            redis,
            postgres,
            rayon,
            argon,
            http,
        }
    }

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
        trace!("spawning blocking task on rayon threadpool");
        let state = self.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.rayon.spawn(move || {
            let _ = tx.send(func(state));
        });
        rx.await
    }

    pub async fn update_user(&self, user: DbUserUpdate) -> Result<User, Error> {
        trace!(?user, "updating user with data");
        let new_db_user = query_as!(
            User,
            "UPDATE users SET
                username = COALESCE($2, username),
                has_stylesheet = COALESCE($3, has_stylesheet),
                biography = COALESCE($4, biography),
                pfp_ext = CASE WHEN $5 THEN NULL ELSE COALESCE($6, pfp_ext) END,
                banner_ext = CASE WHEN $7 THEN NULL ELSE COALESCE($8, banner_ext) END,
                admin = COALESCE($9, admin)
            WHERE id = $1
            RETURNING id, username, has_stylesheet,
            pfp_ext, banner_ext, biography, admin, created_at",
            user.id.get(),
            user.username,
            user.has_stylesheet,
            user.biography,
            user.pfp_ext.is_null(),
            user.pfp_ext.into_option(),
            user.banner_ext.is_null(),
            user.banner_ext.into_option(),
            user.admin
        )
        .fetch_one(&self.postgres)
        .await?;
        trace!(?new_db_user, "updated user with data, adding to redis");
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
        trace!(location, content_type, "creating R2 file");
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
        trace!(location, "deleting R2 file");
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

    pub fn render<T: serde::Serialize>(
        &self,
        template_name: &str,
        data: T,
    ) -> Result<axum::response::Html<String>, Error> {
        let context = tera::Context::from_serialize(data)?;
        self.render_ctx(template_name, &context)
    }

    pub fn render_ctx(
        &self,
        template_name: &str,
        context: &tera::Context,
    ) -> Result<axum::response::Html<String>, Error> {
        trace!(?context, ?template_name, "rendering template");
        #[cfg(feature = "dev")]
        let tera = self
            .tera
            .read()
            .expect("Tera read lock poisoned, check logs for previous panics");
        #[cfg(not(feature = "dev"))]
        let tera = &self.tera;
        let html_text = tera.render(template_name, context)?;
        Ok(axum::response::Html(html_text))
    }

    #[cfg(feature = "dev")]
    pub fn reload_tera(&self) {
        if let Err(source) = self
            .tera
            .write()
            .expect("Tera write lock poisoned, check logs for previous panics!")
            .full_reload()
        {
            if let tera::ErrorKind::Msg(msg) = &source.kind {
                error!("Failed to reload templates: {msg}");
            } else {
                error!(?source, "Failed to reload templates");
            }
        }
    }

    pub async fn from_environment() -> AppState {
        let config: Config = envy::from_env().expect("Failed to read config");
        let root_url = config.root_url.trim_end_matches('/').to_string();
        let cdn_url = config.cdn_url.trim_end_matches('/').to_string();
        let fakes3_endpoint = config.fakes3_endpoint.trim_end_matches('/').to_string();
        let config = Config {
            root_url,
            cdn_url,
            fakes3_endpoint,
            ..config
        };
        let postgres = PgPoolOptions::new()
            .connect(&config.database_url)
            .await
            .expect("Failed to connect to the database");
        sqlx::migrate!().run(&postgres).await.unwrap();
        let redis_mgr = Manager::new(config.redis_url.clone()).expect("failed to connect to redis");
        let redis = deadpool_redis::Pool::builder(redis_mgr)
            .runtime(Runtime::Tokio1)
            .build()
            .unwrap();
        let tera = crate::template::tera();
        let rayon = Arc::new(ThreadPoolBuilder::new().num_threads(8).build().unwrap());
        let argon = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(16384, 192, 8, Some(64)).unwrap(),
        );
        let http = reqwest::ClientBuilder::new()
            .user_agent("speederboard/http")
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Arc::new(InnerAppState::new(
            config.clone(),
            tera,
            redis,
            postgres,
            rayon,
            argon,
            http,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct DbUserUpdate {
    id: Id<UserMarker>,
    username: Option<String>,
    has_stylesheet: Option<bool>,
    biography: Option<String>,
    pfp_ext: MaybeNullUpdate<String>,
    banner_ext: MaybeNullUpdate<String>,
    admin: Option<bool>,
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
            admin: None,
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

    pub fn admin(self, is_admin: bool) -> Self {
        Self {
            admin: Some(is_admin),
            ..self
        }
    }
}

#[derive(Clone, Debug)]
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
