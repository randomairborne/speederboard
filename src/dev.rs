use std::path::Path;

use notify::{Event, Watcher};

use crate::AppState;

pub async fn reload_tera(state: AppState) {
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        trace!(?res, "got notify event");
        if let Ok(event) = res {
            if check_event_interest(&event) {
                let superstate = state.clone();
                info!("reloading templates");
                std::thread::spawn(move || superstate.reload_tera());
            }
        }
    })
    .expect("failed to create watcher");
    watcher
        .watch(Path::new("./templates/"), notify::RecursiveMode::Recursive)
        .expect("Failed to watch for template changes");
    crate::shutdown_signal().await;
}

pub async fn reload_translations(state: AppState) {
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        trace!(?res, "got notify event");
        if let Ok(event) = res {
            if check_event_interest(&event) {
                let superstate = state.clone();
                info!("reloading translations");
                std::thread::spawn(move || superstate.reload_translations());
            }
        }
    })
    .expect("failed to create watcher");
    watcher
        .watch(
            Path::new("./translations/"),
            notify::RecursiveMode::Recursive,
        )
        .expect("Failed to watch for translation changes");
    crate::shutdown_signal().await;
}

fn check_event_interest(event: &Event) -> bool {
    (event.kind.is_modify() || event.kind.is_remove() || event.kind.is_create())
        && (matches!(
            event.kind,
            notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
        ) || matches!(
            event.kind,
            notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
        ))
}

pub async fn cdn_static() {
    let router = axum::Router::new()
        .nest_service("/", tower_http::services::ServeDir::new("./assets/public/"))
        .layer(tower_http::cors::CorsLayer::permissive());
    info!("Starting static CDN on http://localhost:8000");
    axum::Server::bind(&([0, 0, 0, 0], 8000).into())
        .serve(router.into_make_service())
        .with_graceful_shutdown(crate::shutdown_signal())
        .await
        .expect("Failed to start static CDN");
}
