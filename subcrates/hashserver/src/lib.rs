use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::Poll,
};

use http::{HeaderValue, Request, Response, StatusCode};

#[derive(Clone, Debug)]
pub struct Hashserver {
    files: HashMap<String, File>,
    not_found: Arc<[u8]>,
}

#[derive(Clone, Debug)]
pub struct File {
    content: Arc<[u8]>,
    mime: HeaderValue,
}

impl Hashserver {
    pub fn new(dir_asref: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = dir_asref.as_ref().to_path_buf();
        let mut files = HashMap::new();
        let paths = Self::walk(&dir)?;
        for file in paths {
            let content: Arc<[u8]> = std::fs::read(&file)?.into();
            let hash = blake3::hash(&content);
            let hash_str = hash.to_hex();

            let mime = mime_guess::from_path(&file)
                .first_raw()
                .map(HeaderValue::from_static)
                .unwrap_or_else(|| HeaderValue::from_static("application/octet-stream"));

            let raw_serve_path = file.strip_prefix(&dir)?;
            let serve_path = raw_serve_path.to_string_lossy().to_string();
            let path = if let Some(ext) = raw_serve_path
                .extension()
                .map(|v| v.to_string_lossy().to_string())
            {
                let trimmed_serve_path = serve_path.trim_end_matches(&ext);
                format!("/{}-{}.{}", trimmed_serve_path, hash_str, ext)
            } else {
                format!("/{}-{}", serve_path, hash_str)
            };

            let file = File { content, mime };
            files.insert(path, file);
        }
        Ok(Self {
            files,
            not_found: "404 not found".as_bytes().into(),
        })
    }

    fn walk(path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut outputs = Vec::new();
        if path.is_dir() {
            for file in path.read_dir()? {
                let mut children = Self::walk(&file?.path())?;
                outputs.append(&mut children);
            }
        } else if path.is_file() {
            outputs.push(path.clone())
        }
        Ok(outputs)
    }
}

impl tower::Service<Request<()>> for Hashserver {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Response = Response<Arc<[u8]>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<()>) -> Self::Future {
        let Some(file) = self.files.get(req.uri().path()) else {
            let body = self.not_found.clone();
            let fut = async {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "text/plain;charset=utf-8")
                    .body(body)
                    .map_err(Error::Http)
            };
            return Box::pin(fut);
        };
        let body = file.content.clone();
        let mime = file.mime.clone();
        let fut = async {
            Response::builder()
                .header("Content-Type", mime)
                .body(body)
                .map_err(Error::Http)
        };
        Box::pin(fut)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("strip-prefix error: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),
}
