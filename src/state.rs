use crate::{config::Config, Error};
use argon2::Argon2;
use deadpool_redis::{Manager, Pool as RedisPool, Runtime};
use rayon::{ThreadPool, ThreadPoolBuilder};
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
