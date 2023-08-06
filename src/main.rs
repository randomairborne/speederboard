#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod error;
mod id;
mod routes;
mod template;
mod user;

use argon2::Argon2;
use axum::{handler::Handler, routing::get};
use axum_extra::routing::RouterExt;
use deadpool_redis::{Manager, Pool as RedisPool, Runtime};
use rayon::{ThreadPool, ThreadPoolBuilder};
use s3::{creds::Credentials, Bucket};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{fmt::Debug, sync::Arc};
use template::BaseRenderInfo;
use tera::Tera;
use tower_http::{
    compression::CompressionLayer, decompression::DecompressionLayer, services::ServeDir,
};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

pub use crate::error::Error;

pub type AppState = Arc<InnerAppState>;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate sqlx;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let config: Config = envy::from_env().expect("Failed to read config");
    let root_url = config.root_url.trim_end_matches('/').to_string();
    let cdn_url = config.cdn_url.trim_end_matches('/').to_string();
    let config = Config {
        root_url,
        cdn_url,
        ..config
    };
    let postgres = PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to the database");
    sqlx::migrate!().run(&postgres).await.unwrap();
    let redis_mgr = Manager::new(config.redis_url.clone()).expect("failed to connect to redis");
    let redis = RedisPool::builder(redis_mgr)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap();
    let s3 = Bucket::new(
        &config.s3_bucket,
        s3::Region::Custom {
            region: config.s3_region.clone(),
            endpoint: config.s3_endpoint.clone(),
        },
        Credentials::new(
            Some(&config.s3_access_key_id),
            Some(&config.s3_secret_access_key),
            None,
            None,
            None,
        )
        .expect("Invalid S3 credentials"),
    )
    .unwrap()
    .with_path_style();
    let mut tera = Tera::new("./templates/**/*").expect("Failed to build templates");
    tera.register_filter("markdown", |data: &tera::Value, _args: &_| {
        Ok(tera::Value::String(markdown::to_html(&data.to_string())))
    });
    tera.autoescape_on(vec![".html", ".htm", ".jinja", ".jinja2"]);
    let rayon = Arc::new(ThreadPoolBuilder::new().num_threads(8).build().unwrap());
    let argon = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(32767, 8, 8, Some(64)).unwrap(),
    );
    let state = InnerAppState {
        config: config.clone(),
        tera,
        redis,
        postgres,
        rayon,
        argon,
        s3,
    };
    let state = Arc::new(state);
    assert!(
        error::ERROR_STATE.set(state.clone()).is_ok(),
        "Could not set error state, this should be impossible"
    );
    info!("Starting server on http://localhost:{}", config.port);
    axum::Server::bind(&([0, 0, 0, 0], config.port).into())
        .serve(build_router(state).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn build_router(state: AppState) -> axum::Router {
    let servedir =
        ServeDir::new("./public/").fallback(routes::notfound_handler.with_state(state.clone()));
    axum::Router::new()
        .route("/", get(routes::index::get))
        .route_with_tsr("/login", get(routes::login::get).post(routes::login::post))
        .route_with_tsr(
            "/signup",
            get(routes::signup::get).post(routes::signup::post),
        )
        .route_with_tsr(
            "/user/:username",
            get(routes::user::get).put(routes::user::put),
        )
        .route(
            "/user/:username/assets",
            get(routes::user::presigns).put(routes::user::extensions),
        )
        .layer(CompressionLayer::new())
        .layer(DecompressionLayer::new())
        .fallback_service(servedir)
        .with_state(state)
}

async fn shutdown_signal() {
    #[cfg(not(target_family = "unix"))]
    compile_error!("WASM and windows are not supported platforms, please use WSL if on windows!");
    #[cfg(target_family = "unix")]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut interrupt = signal(SignalKind::interrupt()).expect("Failed to listen to sigint");
        let mut quit = signal(SignalKind::quit()).expect("Failed to listen to sigquit");
        let mut terminate = signal(SignalKind::terminate()).expect("Failed to listen to sigterm");

        tokio::select! {
            _ = interrupt.recv() => {},
            _ = quit.recv() => {},
            _ = terminate.recv() => {}
        }
    }
}

#[derive(Clone)]
pub struct InnerAppState {
    config: Config,
    tera: Tera,
    redis: RedisPool,
    postgres: PgPool,
    rayon: Arc<ThreadPool>,
    argon: Argon2<'static>,
    s3: Bucket,
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
}

#[derive(serde::Deserialize, Clone, Debug)]
struct Config {
    redis_url: String,
    database_url: String,
    root_url: String,
    cdn_url: String,
    s3_endpoint: String,
    s3_bucket: String,
    s3_region: String,
    s3_secret_access_key: String,
    s3_access_key_id: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_port() -> u16 {
    8080
}
