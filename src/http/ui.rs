//! Brain UI assets served at `GET /ui` — single product surface.
//!
//! Source files live under `ui/` (HTML/CSS/JS/images). They are embedded at
//! compile time so the binary does not depend on a Vite process or a filesystem
//! layout next to the executable. Optional Vite preview lives under `website/`
//! and is not a second product.

use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "ui/"]
struct UiAssets;

/// Routes for the daemon brain UI (`/ui` → embedded `ui/` assets).
pub fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/ui", get(|| async { Redirect::permanent("/ui/") }))
        .route("/ui/", get(|| async { serve_asset("brain.html") }))
        .route("/ui/{*path}", get(ui_path))
}

// Old marketing landing page — disabled, redirect to brain.
const LEGACY_PATHS: &[&str] = &["index.html", "index.js", "index.css"];

async fn ui_path(Path(path): Path<String>) -> Response {
    let clean = path.trim_start_matches('/');
    if LEGACY_PATHS.contains(&clean) {
        return Redirect::permanent("/ui/").into_response();
    }
    serve_asset(clean)
}

fn serve_asset(path: &str) -> Response {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return serve_asset("brain.html");
    }
    if path.contains("..") || path.starts_with('/') {
        return StatusCode::NOT_FOUND.into_response();
    }

    match UiAssets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let cache = if path.starts_with("assets/") {
                "public, max-age=31536000, immutable"
            } else {
                "no-store"
            };
            (
                [
                    (header::CONTENT_TYPE, mime.essence_str()),
                    (header::CACHE_CONTROL, cache),
                ],
                file.data.into_owned(),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn ui_root_serves_brain_html() {
        let app = routes::<()>();
        let resp = app
            .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.starts_with("text/html"), "content-type={ct}");
        let cache = resp
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert_eq!(cache, "no-store");
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body = String::from_utf8_lossy(&bytes);
        assert!(
            body.contains("Brain") || body.contains("kurultai"),
            "expected brain UI html"
        );
        assert!(
            body.contains("kurultai-ui-version"),
            "brain.html should stamp UI version"
        );
    }

    #[tokio::test]
    async fn ui_redirects_bare_path() {
        let app = routes::<()>();
        let resp = app
            .oneshot(Request::builder().uri("/ui").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(
            resp.status().is_redirection(),
            "expected redirect, got {}",
            resp.status()
        );
        let loc = resp
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert_eq!(loc, "/ui/");
    }

    #[tokio::test]
    async fn ui_serves_css_asset() {
        // Legacy flat assets (index.html/index.js/index.css) redirect to /ui/.
        let app = routes::<()>();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/ui/index.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            resp.status().is_redirection(),
            "legacy index.css should redirect, got {}",
            resp.status()
        );

        // The real CSS bundle lives under assets/ with a content hash — find it.
        let css_path = UiAssets::iter()
            .find(|p| p.starts_with("assets/") && p.ends_with(".css"))
            .map(|p| p.into_owned())
            .expect("a hashed css bundle must be embedded");
        let app = routes::<()>();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/ui/{css_path}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.contains("css"), "content-type={ct}");
        let cache = resp
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            cache.contains("immutable"),
            "hashed assets should be immutable, got {cache}"
        );
    }
}
