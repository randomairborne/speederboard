use axum::{
    routing::{get, post},
    Router,
};
use axum_extra::routing::RouterExt;
use tower_http::{compression::CompressionLayer, decompression::DecompressionLayer};

use crate::{routes, AppState};

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/", get(routes::index::get))
        .route_with_tsr("/login", get(routes::login::get).post(routes::login::post))
        .route_with_tsr("/logout", get(routes::login::logout))
        .route_with_tsr(
            "/signup",
            get(routes::signup::get).post(routes::signup::post),
        )
        .route_with_tsr("/user/:username", get(routes::user::get))
        .merge(game_router(state.clone()))
        .merge(settings_router(state.clone()))
        .merge(admin_router(state.clone()))
        .layer(CompressionLayer::new())
        .layer(DecompressionLayer::new())
        .fallback(routes::notfound_handler)
        .with_state(state)
}

pub fn settings_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route_with_tsr(
            "/settings",
            get(routes::settings::get).post(routes::settings::profile),
        )
        .route("/settings/pfp", post(routes::settings::files::pfp))
        .route(
            "/settings/pfp/delete",
            post(routes::settings::files::pfp_del),
        )
        .route("/settings/banner", post(routes::settings::files::banner))
        .route(
            "/settings/banner/delete",
            post(routes::settings::files::banner_del),
        )
        .route(
            "/settings/stylesheet",
            post(routes::settings::files::stylesheet),
        )
        .route(
            "/settings/stylesheet/delete",
            post(routes::settings::files::stylesheet_del),
        )
        .route(
            "/settings/email",
            post(routes::settings::credentials::update_email),
        )
        .route(
            "/settings/password",
            post(routes::settings::credentials::update_password),
        )
        .with_state(state)
}

pub fn game_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route_with_tsr("/game/:gameslug", get(routes::game::index::get))
        .route_with_tsr(
            "/game/:gameslug/category/:catid",
            get(routes::game::category::get),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/run/new",
            get(routes::game::new_run::get).post(routes::game::new_run::create),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/run/:runid",
            get(routes::game::run::get),
        )
        .with_state(state)
}

pub fn admin_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route_with_tsr(
            "/admin/newgame",
            get(routes::admin::game::get).post(routes::admin::game::post),
        )
        .with_state(state)
}
