use std::{num::NonZeroUsize, sync::Arc, time::Duration};

use argon2::Argon2;
use axum::{http::HeaderValue, response::Redirect};
use deadpool_redis::{Manager, Pool as RedisPool, Runtime};
use rayon::{ThreadPool, ThreadPoolBuilder};
use redis::AsyncCommands;
use s3::creds::Credentials;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tera::Tera;
use url::Url;

use crate::{config::Config, Error};

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
    bucket: s3::Bucket,
    csp: HeaderValue,
}

impl InnerAppState {
    const DEFAULT_THREADPOOL_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(8) };

    pub fn new(
        config: Config,
        tera: InnerTera,
        redis: RedisPool,
        postgres: PgPool,
        rayon: Arc<ThreadPool>,
        argon: Argon2<'static>,
        http: reqwest::Client,
        bucket: s3::Bucket,
        csp: HeaderValue,
    ) -> Self {
        Self {
            config,
            tera,
            redis,
            postgres,
            rayon,
            argon,
            http,
            bucket,
            csp,
        }
    }

    /// # Errors
    /// If somehow the channel hangs up, this can error.
    pub async fn spawn_rayon<O, F, S>(
        &self,
        func: F,
        state: S,
    ) -> Result<O, tokio::sync::oneshot::error::RecvError>
    where
        O: Send + 'static,
        S: Send + 'static,
        F: Fn(InnerAppState, S) -> O + Send + 'static,
    {
        trace!("spawning blocking task on rayon threadpool");
        let app_state = self.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.rayon.spawn(move || {
            let _ = tx.send(func(app_state, state));
        });
        rx.await
    }

    pub async fn put_r2_file(
        &self,
        location: &str,
        file: &[u8],
        content_type: &str,
    ) -> Result<(), Error> {
        trace!(location, content_type, "creating R2 file");
        let resp = self
            .bucket
            .put_object_with_content_type(location, file, content_type)
            .await?;
        trace!(?resp, "got response on file creation");
        Self::s3_status_success(resp.status_code())?;
        Ok(())
    }

    pub async fn delete_r2_file(&self, location: &str) -> Result<(), Error> {
        trace!(location, "deleting R2 file");
        let resp = self.bucket.delete_object(location).await?;
        trace!(?resp, "got response on file deletion");
        Self::s3_status_success(resp.status_code())?;
        Ok(())
    }

    fn s3_status_success(status: u16) -> Result<(), Error> {
        if (200u16..300u16).contains(&status) {
            Ok(())
        } else {
            Err(Error::S3Status(status))
        }
    }

    pub fn redirect(&self, location: impl AsRef<str>) -> Redirect {
        let root = &self.config.root_url;
        let path = location.as_ref();
        Redirect::to(&format!("{root}{path}"))
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

    pub fn csp(&self) -> HeaderValue {
        self.csp.clone()
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

    #[cfg(feature = "dev")]
    pub fn reload_translations(&self) {
        let translations = match crate::template::get_translations() {
            Ok(v) => v,
            Err(source) => {
                error!(?source, "Failed to reload translations");
                return;
            }
        };
        self.tera
            .write()
            .expect("Tera write lock poisoned, check logs for previous panics!")
            .register_function(
                "gettrans",
                crate::template::GetTranslation::new(translations),
            );
    }

    async fn get_s3_bucket_from_config(config: &Config) -> s3::Bucket {
        let region = if let Some(account_id) = config.r2_account_id.clone() {
            s3::Region::R2 { account_id }
        } else {
            s3::Region::Custom {
                region: config
                    .s3_region
                    .clone()
                    .expect("Must have a S3_REGION in config if R2_ACCOUNT_ID is not present!"),
                endpoint: config
                    .s3_endpoint
                    .clone()
                    .expect("Must have a S3_ENDPOINT in config if R2_ACCOUNT_ID is not present!")
                    .trim_end_matches('/')
                    .to_owned(),
            }
        };
        let mut bucket = s3::Bucket::new(
            &config.s3_bucket_name,
            region,
            Credentials::new(
                config.s3_access_key.as_deref(),
                config.s3_secret_key.as_deref(),
                None,
                None,
                None,
            )
            .expect("Invalid S3 credentials"),
        )
        .expect("Invalid bucket (this is a bug)");
        if config.s3_path_style {
            bucket.set_path_style();
        }
        bucket
    }

    fn url_to_origin(input: &str) -> String {
        Url::parse(input)
            .expect("ROOT_URL is not a valid URL")
            .origin()
            .ascii_serialization()
    }

    pub async fn get_redis_object<
        T: for<'de> serde::Deserialize<'de>,
        K: redis::ToRedisArgs + Send + Sync,
    >(
        &self,
        key: K,
    ) -> Result<Option<T>, Error> {
        let maybe_object_str: Option<String> = self.redis.get().await?.get(key).await?;
        if let Some(object_str) = maybe_object_str {
            let object: T = serde_json::from_str(&object_str)?;
            Ok(Some(object))
        } else {
            Ok(None)
        }
    }

    pub async fn set_redis_object<K: redis::ToRedisArgs + Send + Sync, V: serde::Serialize>(
        &self,
        key: K,
        data: &V,
        expiry: usize,
    ) -> Result<(), Error> {
        let game_str = serde_json::to_string(data)?;
        self.redis
            .get()
            .await?
            .set_ex(key, game_str, expiry)
            .await?;
        Ok(())
    }

    #[cfg(test)]
    pub async fn test(db: PgPool) -> AppState {
        let mut me = Self::inner_from_environment().await;
        me.postgres = db;
        Arc::new(me)
    }

    pub async fn from_environment() -> AppState {
        Arc::new(Self::inner_from_environment().await)
    }

    pub async fn inner_from_environment() -> Self {
        let config: Config = envy::from_env().expect("Failed to read config");
        let root_url = config.root_url.trim_end_matches('/').to_string();
        let static_url = config.static_url.trim_end_matches('/').to_string();
        let user_content_url = config.user_content_url.trim_end_matches('/').to_string();
        let config = Config {
            root_url,
            static_url,
            user_content_url,
            ..config
        };
        let csp = HeaderValue::from_str(&format!(
            "default-src {0} {1} {2}; script-src {1}; object-src 'none'; \
            frame-src https://youtube.com https://clips.twitch.tv {2}",
            Self::url_to_origin(&config.root_url),
            Self::url_to_origin(&config.static_url),
            Self::url_to_origin(&config.user_content_url)
        ))
        .expect("Invalid csp header value (check your STATIC_URL)");
        let postgres = PgPoolOptions::new()
            .max_connections(15)
            .connect(&config.database_url)
            .await
            .expect("Failed to connect to the database");
        sqlx::migrate!().run(&postgres).await.unwrap();
        let redis_mgr = Manager::new(config.redis_url.clone()).expect("failed to connect to redis");
        let redis = deadpool_redis::Pool::builder(redis_mgr)
            .runtime(Runtime::Tokio1)
            .build()
            .unwrap();
        redis.get().await.expect("Failed to load redis");
        let tera = crate::template::tera(&config);
        let rayon = Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(
                    std::thread::available_parallelism()
                        .unwrap_or(Self::DEFAULT_THREADPOOL_SIZE)
                        .get(),
                )
                .build()
                .unwrap(),
        );
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
        let bucket = Self::get_s3_bucket_from_config(&config).await;
        InnerAppState::new(
            config.clone(),
            tera,
            redis,
            postgres,
            rayon,
            argon,
            http,
            bucket,
            csp,
        )
    }
}
