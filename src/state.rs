use std::{
    collections::HashMap,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use argon2::Argon2;
use axum::{http::HeaderValue, response::Redirect};
use deadpool_redis::{Manager, Pool as RedisPool, Runtime};
use parking_lot::RwLock;
use rayon::{ThreadPool, ThreadPoolBuilder};
use redis::AsyncCommands;
use s3::creds::Credentials;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tera::Tera;
use url::Url;

use crate::{config::Config, Error};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    tera: Arc<RwLock<Tera>>,
    pub redis: RedisPool,
    pub postgres: PgPool,
    rayon: Arc<ThreadPool>,
    pub argon: Arc<Argon2<'static>>,
    pub http: reqwest::Client,
    pub static_hashes: Arc<RwLock<HashMap<String, String>>>,
    bucket: Arc<s3::Bucket>,
    csp: Arc<HeaderValue>,
}

impl AppState {
    const DEFAULT_THREADPOOL_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(8) };

    pub fn new(
        config: Arc<Config>,
        tera: Arc<RwLock<Tera>>,
        redis: RedisPool,
        postgres: PgPool,
        rayon: Arc<ThreadPool>,
        argon: Arc<Argon2<'static>>,
        http: reqwest::Client,
        static_hashes: Arc<RwLock<HashMap<String, String>>>,
        bucket: Arc<s3::Bucket>,
        csp: Arc<HeaderValue>,
    ) -> Self {
        Self {
            config,
            tera,
            redis,
            postgres,
            rayon,
            argon,
            http,
            static_hashes,
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
        F: Fn(AppState, S) -> O + Send + 'static,
    {
        trace!("spawning blocking task on rayon threadpool");
        let (tx, rx) = tokio::sync::oneshot::channel();
        let app_state = self.clone();
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
        let html_text = self.tera.read().render(template_name, context)?;
        Ok(axum::response::Html(html_text))
    }

    pub fn csp(&self) -> HeaderValue {
        (*self.csp).clone()
    }

    #[cfg(feature = "dev")]
    pub fn reload_tera(&self) {
        if let Err(source) = self.tera.write().full_reload() {
            if let tera::ErrorKind::Msg(msg) = &source.kind {
                error!("Failed to reload templates: {msg}");
            } else {
                error!(?source, "Failed to reload templates");
            }
        }
    }

    #[cfg(feature = "dev")]
    pub fn reload_assets(&self) {
        let new_hashes = Self::walk_for_hashes("./assets/public/");
        *self.static_hashes.write() = new_hashes;
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
        self.tera.write().register_function(
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

    pub fn static_resource(&self, path: &str) -> String {
        let map = self.static_hashes.read();
        let bust = map.get(path).map_or("none", |v| v.as_str());
        format!("{}/static{}?cb={bust}", self.config.root_url, path)
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
        let mut me = Self::from_environment().await;
        me.postgres = db;
        me
    }

    pub async fn from_environment() -> AppState {
        let config: Config = envy::from_env().expect("Failed to read config");
        let root_url = config.root_url.trim_end_matches('/').to_string();
        let user_content_url = config.user_content_url.trim_end_matches('/').to_string();
        let config = Arc::new(Config {
            root_url,
            user_content_url,
            ..config
        });
        let csp = Arc::new(
            HeaderValue::from_str(&format!(
                "default-src {0} {1}; script-src {0}/static/page-scripts/; \
                frame-src https://youtube.com https://clips.twitch.tv {1}; \
                require-trusted-types-for 'script'; object-src 'none';",
                Self::url_to_origin(&config.root_url),
                Self::url_to_origin(&config.user_content_url),
            ))
            .expect("Invalid csp header value (check your USER_CONTENT_URL)"),
        );
        let static_hashes = Arc::new(RwLock::new(Self::walk_for_hashes("./assets/public/")));
        trace!(?static_hashes, "static hashes created");
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
        let argon = Arc::new(Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(16384, 192, 8, Some(64)).unwrap(),
        ));

        let http = reqwest::ClientBuilder::new()
            .user_agent("speederboard/http")
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        let bucket = Arc::new(Self::get_s3_bucket_from_config(&config).await);
        let tera = Arc::new(RwLock::new(Tera::default()));
        let temp_state = AppState::new(
            config,
            tera,
            redis,
            postgres,
            rayon,
            argon,
            http,
            static_hashes,
            bucket,
            csp,
        );
        *temp_state.clone().tera.write() = crate::template::tera(temp_state.clone());
        temp_state
    }

    fn walk_for_hashes(path: impl AsRef<Path>) -> HashMap<String, String> {
        let path = path.as_ref();
        let mut output = HashMap::new();
        let files = Self::walkdir(path).expect("Failed to get cachebusting files");
        for file in files {
            let data = std::fs::read(&file).expect("Failed to read file");
            let path = file
                .strip_prefix(path)
                .expect("Failed to strip prefix of file")
                .to_str()
                .expect("Bad characters in path");
            let hash = blake3::hash(&data);
            output.insert(format!("/{path}"), hash.to_hex().to_string());
        }
        output
    }

    fn walkdir(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut outputs = Vec::new();
        for file in path.read_dir()? {
            let file = file?;
            let kind = file.file_type()?;
            if kind.is_dir() {
                let mut children = Self::walkdir(&file.path())?;
                outputs.append(&mut children);
            } else if kind.is_file() {
                outputs.push(file.path())
            }
        }
        trace!(paths=?outputs, parent=?path, "walked parent for paths");
        Ok(outputs)
    }
}
