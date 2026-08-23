//! 内嵌前端静态资源：release 编译进二进制，debug 从 web/dist 实时读盘

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
            // mime_guess 产出恒为 ASCII，解析失败就留给浏览器嗅探
            if let Ok(v) = HeaderValue::from_str(file.metadata.mimetype()) {
                headers.insert(header::CONTENT_TYPE, v);
            }
            // assets/ 文件名带内容哈希可长缓存；其余（index.html 等）不缓存
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
        // SPA 回退：无扩展名的路径交给前端路由
        None if !path.contains('.') => serve("index.html"),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
