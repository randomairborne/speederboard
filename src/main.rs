mod config;
mod error;
mod id;
mod language;
mod model;
mod router;
mod routes;
mod state;
mod template;
mod util;

#[cfg(feature = "dev")]
mod dev;

#[cfg(test)]
mod test;

use std::net::SocketAddr;

use axum::response::Html;
use tokio::net::TcpListener;

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
    util::start_tracing();
    let state = AppState::from_environment().await;
    #[cfg(feature = "dev")]
    let (tera_jh, translations_jh, assets_jh) = {
        let tera_jh = tokio::spawn(dev::reload_tera(state.clone()));
        let translations_jh = tokio::spawn(dev::reload_translations(state.clone()));
        let assets_jh = tokio::spawn(dev::reload_assets(state.clone()));
        (tera_jh, translations_jh, assets_jh)
    };
    let bind_address = SocketAddr::from(([0, 0, 0, 0], state.config.port));
    info!("Starting server on http://localhost:{}", state.config.port);
    let app = router::build(state);
    let tcp = TcpListener::bind(bind_address).await.unwrap();
    axum::serve(tcp, app)
        .with_graceful_shutdown(vss::shutdown_signal())
        .await
        .unwrap();

    #[cfg(feature = "dev")]
    {
        tera_jh.await.unwrap();
        translations_jh.await.unwrap();
        assets_jh.await.unwrap();
    }
}
