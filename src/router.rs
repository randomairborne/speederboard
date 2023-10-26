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
        .merge(settings_router(state.clone()))
        .merge(game_router(state.clone()))
        .merge(forum_router(state.clone()))
        .merge(admin_router(state.clone()))
        .layer(
            tower::ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    crate::error::error_middleware,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    crate::util::csp_middleware,
                ))
                .layer(CompressionLayer::new())
                .layer(DecompressionLayer::new()),
        )
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
        .route_with_tsr(
            "/game/:gameslug",
            get(routes::game::category::default_category),
        )
        .route_with_tsr(
            "/game/:gameslug/edit",
            get(routes::game::settings::game::get).post(routes::game::settings::game::edit),
        )
        .route_with_tsr(
            "/game/:gameslug/edit/new-category",
            post(routes::game::settings::category::new),
        )
        .route_with_tsr(
            "/game/:gameslug/feed",
            get(routes::game::modtools::feed::game_feed),
        )
        .route_with_tsr(
            "/game/:gameslug/team",
            get(routes::game::modtools::team::get).post(routes::game::modtools::team::post),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/edit/makedefault",
            post(routes::game::settings::game::set_default_category),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/edit/delete",
            get(routes::game::settings::category::confirm_delete)
                .post(routes::game::settings::category::delete),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid",
            get(routes::game::category::specific_category),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/feed",
            get(routes::game::modtools::feed::category_feed),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/edit",
            get(routes::game::settings::category::get).post(routes::game::settings::category::edit),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/run/new",
            get(routes::game::new_run::get).post(routes::game::new_run::create),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/run/:runid",
            get(routes::game::run::get),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/run/:runid/review",
            get(routes::game::modtools::run::fetch_review),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/run/:runid/verify",
            post(routes::game::modtools::run::verify_run),
        )
        .route_with_tsr(
            "/game/:gameslug/category/:catid/run/:runid/reject",
            post(routes::game::modtools::run::reject_run),
        )
        .with_state(state)
}

pub fn forum_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route_with_tsr("/forum/:gameslug", get(routes::forum::root::get))
        .route_with_tsr(
            "/forum/:gameslug/new",
            get(routes::forum::new_post::get).post(routes::forum::new_post::post),
        )
        .route_with_tsr(
            "/forum/:gameslug/post/:postid",
            get(routes::forum::post::get).post(routes::forum::post::post),
        )
        .with_state(state)
}

pub fn admin_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route_with_tsr(
            "/admin/newgame",
            get(routes::admin::game::get).post(routes::admin::game::post),
        )
        .route_with_tsr("/admin/inspect/user/:id", get(|| async {}))
        .with_state(state)
}
