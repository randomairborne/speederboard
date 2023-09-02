#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod config;
mod error;
mod id;
mod model;
mod router;
mod routes;
mod state;
mod template;
mod util;

#[cfg(feature = "dev")]
mod dev;

use argon2::Argon2;
use axum::response::Html;
use deadpool_redis::{Manager, Pool as RedisPool, Runtime};
use rayon::ThreadPoolBuilder;
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::{config::Config, state::InnerAppState};
pub use crate::{error::Error, state::AppState};

pub type HandlerResult = Result<Html<String>, Error>;

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
    let redis = RedisPool::builder(redis_mgr)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap();
    let tera = template::tera();
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
    let state = Arc::new(InnerAppState::new(
        config.clone(),
        tera,
        redis,
        postgres,
        rayon,
        argon,
        http,
    ));
    #[cfg(feature = "dev")]
    let (tera_jh, cdn_jh, fakes3_jh) = {
        let s2 = state.clone();
        let tera_jh = tokio::spawn(crate::dev::reload_tera(s2));
        let cdn_jh = tokio::spawn(crate::dev::cdn());
        let fakes3_jh = tokio::spawn(crate::dev::fakes3());
        (tera_jh, cdn_jh, fakes3_jh)
    };
    info!("Starting server on http://localhost:{}", config.port);
    axum::Server::bind(&([0, 0, 0, 0], config.port).into())
        .serve(router::build(state).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    #[cfg(feature = "dev")]
    {
        cdn_jh.await.unwrap();
        fakes3_jh.await.unwrap();
        tera_jh.await.unwrap();
    }
}

async fn shutdown_signal() {
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
    #[cfg(not(target_family = "unix"))]
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen to ctrl+c");
}
