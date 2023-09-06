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

use axum::response::Html;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::state::InnerAppState;
pub use crate::{error::Error, state::AppState};

pub type HandlerResult = Result<Html<String>, Error>;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate sqlx;

#[cfg(feature = "dev")]
pub const DEV_MODE: bool = true;

#[cfg(not(feature = "dev"))]
pub const DEV_MODE: bool = false;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(concat!(env!("CARGO_PKG_NAME"), "=info").parse().unwrap())
        .with_env_var("LOG")
        .from_env()
        .expect("failed to parse env");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .init();
    let state = InnerAppState::from_environment().await;
    #[cfg(feature = "dev")]
    let (tera_jh, cdn_jh, fakes3_jh) = {
        let s2 = state.clone();
        let tera_jh = tokio::spawn(crate::dev::reload_tera(s2));
        let cdn_jh = tokio::spawn(crate::dev::cdn());
        let fakes3_jh = tokio::spawn(crate::dev::fakes3());
        (tera_jh, cdn_jh, fakes3_jh)
    };
    info!("Starting server on http://localhost:{}", state.config.port);
    axum::Server::bind(&([0, 0, 0, 0], state.config.port).into())
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
