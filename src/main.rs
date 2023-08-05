#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod error;
mod id;
mod routes;
mod template;
mod user;

use argon2::Argon2;
use axum::{handler::Handler, routing::get};
use deadpool_redis::{Manager, Pool as RedisPool, Runtime};
use rayon::{ThreadPool, ThreadPoolBuilder};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use template::BaseRenderInfo;
use tera::Tera;
use tokio::signal::unix::SignalKind;
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
    let mut tera = Tera::new("./templates/**/*.jinja").expect("Failed to build templates");
    tera.autoescape_on(vec![".html", ".htm", ".jinja", ".jinja2"]);
    error::ERROR_TERA.set(tera.clone()).unwrap();
    let rayon = Arc::new(ThreadPoolBuilder::new().num_threads(8).build().unwrap());
    let argon = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(32767, 8, 2, Some(128)).unwrap(),
    );
    let state = InnerAppState {
        config: config.clone(),
        tera,
        redis,
        postgres,
        rayon,
        argon,
    };
    let state = Arc::new(state);
    let servedir = ServeDir::new("./public/").fallback(routes::notfound.with_state(state.clone()));
    let router = axum::Router::new()
        .route("/login", get(routes::login::page).post(routes::login::form))
        .route(
            "/signup",
            get(routes::signup::page).post(routes::signup::form),
        )
        .layer(CompressionLayer::new())
        .layer(DecompressionLayer::new()).nest_service("/", servedir)
        .with_state(state);
    info!("Starting server on http://localhost:{}", config.port);
    axum::Server::bind(&([0, 0, 0, 0], config.port).into())
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    #[cfg(not(target_family = "unix"))]
    compile_error!("WASM and windows are not supported platforms, please use WSL if on windows!");
    #[cfg(target_family = "unix")]
    {
        use tokio::signal::unix::signal;
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
}

impl InnerAppState {
    async fn spawn_rayon<O, F>(&self, func: F) -> Result<O, tokio::sync::oneshot::error::RecvError>
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
    fn base_context(&self) -> BaseRenderInfo {
        BaseRenderInfo::new(&self.config.root_url, &self.config.cdn_url)
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
struct Config {
    redis_url: String,
    database_url: String,
    root_url: String,
    cdn_url: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_port() -> u16 {
    8080
}
