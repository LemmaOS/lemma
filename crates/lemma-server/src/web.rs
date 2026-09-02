//! Serves the web build embedded at compile time.

use axum::{
    body::Body,
    http::{Response, StatusCode, Uri, header, header::HeaderValue},
    response::IntoResponse,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../web/dist"]
struct WebDist;

pub async fn handler(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    serve(if path.is_empty() { "index.html" } else { path })
}

fn serve(path: &str) -> Response<Body> {
    match WebDist::get(path) {
        Some(file) => {
            let mut res = Response::new(Body::from(file.data.into_owned()));
            let headers = res.headers_mut();
            if let Ok(v) = HeaderValue::from_str(file.metadata.mimetype()) {
                headers.insert(header::CONTENT_TYPE, v);
            }
            // assets/ files have content-hashed names and can be cached
            // forever; everything else must revalidate so a new deploy
            // is picked up.
            headers.insert(
                header::CACHE_CONTROL,
                if path.starts_with("assets/") {
                    HeaderValue::from_static("public, max-age=31536000, immutable")
                } else {
                    HeaderValue::from_static("no-cache")
                },
            );
            res
        }
        // Paths without an extension are client-side routes; hand them
        // to the SPA.
        None if !path.contains('.') => serve("index.html"),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
