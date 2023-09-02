use std::path::Path;

use axum::{body::Bytes, http::Uri};
use notify::{Event, Watcher};

use crate::{AppState, Error};

pub fn reload_tera(state: AppState) {
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if !event.kind.is_modify() || !event.kind.is_remove() || !event.kind.is_create() {
                return;
            }
            let superstate = state.clone();
            std::thread::spawn(move || superstate.reload_tera());
        }
    })
    .expect("failed to create watcher");
    watcher
        .watch(Path::new("./templates/"), notify::RecursiveMode::Recursive)
        .expect("Failed to watch for template changes");
}

pub async fn cdn() {
    let router = axum::Router::new()
        .route_service("/", tower_http::services::ServeDir::new("./assets/public/"))
        .layer(tower_http::cors::CorsLayer::permissive());
    axum::Server::bind(&([0, 0, 0, 0], 8000).into())
        .serve(router.into_make_service())
        .with_graceful_shutdown(crate::shutdown_signal())
        .await
        .expect("Failed to start CDN");
}

pub async fn fakes3() {
    let router = axum::Router::new().route("/*", axum::routing::put(put).delete(delete));
    axum::Server::bind(&([0, 0, 0, 0], 8001).into())
        .serve(router.into_make_service())
        .with_graceful_shutdown(crate::shutdown_signal())
        .await
        .expect("Failed to start fakeS3");
}

async fn put(uri: Uri, body: Bytes) -> Result<(), Error> {
    let uri_path = uri.path();
    if uri_path.contains("..") {
        return Err(Error::DoubleDotInPath);
    }
    let path = Path::new(&format!("./assets/public{uri_path}")).canonicalize()?;
    tokio::fs::create_dir_all(path.parent().ok_or(Error::PathHasNoParent)?).await?;
    tokio::fs::write(path, body).await?;
    Ok(())
}

async fn delete(uri: Uri) -> Result<(), Error> {
    let uri_path = uri.path();
    if uri_path.contains("..") {
        return Err(Error::DoubleDotInPath);
    }
    let path = Path::new(&format!("./assets/public{uri_path}")).canonicalize()?;
    tokio::fs::remove_file(path).await?;
    Ok(())
}
