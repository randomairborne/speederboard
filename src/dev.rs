use std::path::Path;

use axum::{body::Bytes, http::Uri};
use notify::{Event, Watcher};

use crate::{AppState, Error};

pub fn reload_tera(state: AppState) {
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        trace!(?res, "got watch notification");
        if let Ok(event) = res {
            if !event.kind.is_modify() || !event.kind.is_remove() || !event.kind.is_create() {
                return;
            }
            let superstate = state.clone();
            debug!("reloading templates");
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
        .nest_service(
            "/",
            tower_http::services::ServeDir::new("./assets/")
        )
        .layer(tower_http::cors::CorsLayer::permissive());
    info!("Starting CDN on http://localhost:8000");
    axum::Server::bind(&([0, 0, 0, 0], 8000).into())
        .serve(router.into_make_service())
        .with_graceful_shutdown(crate::shutdown_signal())
        .await
        .expect("Failed to start CDN");
}

pub async fn fakes3() {
    let router = axum::Router::new().route("/*unused", axum::routing::put(put).delete(delete));
    info!("Starting FakeS3 on http://localhost:8001");
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
    let path_string = format!("./assets{uri_path}");
    let path = Path::new(&path_string);
    let parent = path.parent().ok_or(Error::PathHasNoParent)?;
    trace!("Got request to {uri_path} (updated to {path_string})");
    tokio::fs::create_dir_all(parent).await?;
    tokio::fs::write(&path, body).await?;
    trace!("Created file {}", path.to_string_lossy());
    Ok(())
}

async fn delete(uri: Uri) -> Result<(), Error> {
    let uri_path = uri.path();
    if uri_path.contains("..") {
        return Err(Error::DoubleDotInPath);
    }
    let path_string = format!("./assets{uri_path}");
    let path = Path::new(&path_string);
    trace!("Got request to {uri_path} (updated to {path_string})");
    tokio::fs::remove_file(&path).await?;
    trace!("Removed file {}", path.to_string_lossy());
    Ok(())
}
