//! UI static file server with embedded assets.
//!
//! Serves the Vue 3 dashboard UI as embedded static files.
//! Assets are embedded at compile time using rust-embed.

use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use rust_embed::Embed;
use std::net::SocketAddr;

/// Embedded UI assets from the Vue build output.
///
/// Assets are embedded from `../openclaw-ui/dist` at compile time.
/// The `compression` feature compresses assets for smaller binary size.
#[derive(Embed)]
#[folder = "../openclaw-ui/dist"]
#[prefix = ""]
struct UiAssets;

/// UI server configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UiServerConfig {
    /// Port for UI server (default: 3000).
    pub port: u16,
    /// Bind address (default: 127.0.0.1).
    pub bind_address: String,
    /// Whether the UI server is enabled.
    pub enabled: bool,
}

impl Default for UiServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            bind_address: "127.0.0.1".to_string(),
            enabled: true,
        }
    }
}

impl UiServerConfig {
    /// Create a new UI server config with custom port.
    #[must_use]
    pub fn with_port(port: u16) -> Self {
        Self {
            port,
            ..Default::default()
        }
    }

    /// Get the full address string.
    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}

/// Serve the root index.html.
async fn serve_index() -> Response {
    serve_file("index.html").await
}

/// Serve embedded static files with SPA fallback.
async fn serve_static(Path(path): Path<String>) -> Response {
    serve_file(&path).await
}

/// Internal file serving logic.
///
/// Tries to serve the requested file from embedded assets.
/// If the file is not found and doesn't have an extension (not a file),
/// falls back to serving index.html for SPA routing support.
async fn serve_file(path: &str) -> Response {
    // Try to get the file from embedded assets
    if let Some(content) = UiAssets::get(path) {
        // Determine MIME type from file extension
        let mime = mime_guess::from_path(path).first_or_octet_stream();

        // Set cache headers based on file type
        let cache_control = if path.contains(".js")
            || path.contains(".css")
            || path.contains(".woff")
            || path.contains(".woff2")
        {
            // Long cache for hashed assets
            "public, max-age=31536000, immutable"
        } else if path == "index.html" {
            // No cache for index.html to ensure fresh content
            "no-cache, no-store, must-revalidate"
        } else {
            // Short cache for other assets
            "public, max-age=3600"
        };

        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, mime.as_ref()),
                (header::CACHE_CONTROL, cache_control),
            ],
            content.data.into_owned(),
        )
            .into_response()
    } else {
        // SPA fallback: serve index.html for non-file routes
        // A route is considered a file if it contains a dot (e.g., .js, .css, .png)
        if !path.contains('.') {
            if let Some(index) = UiAssets::get("index.html") {
                return (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "text/html"),
                        (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                    ],
                    index.data.into_owned(),
                )
                    .into_response();
            }
        }

        // File not found
        (StatusCode::NOT_FOUND, "Not Found").into_response()
    }
}

/// Create the UI router.
///
/// Sets up routes for serving static files with SPA fallback.
pub fn create_ui_router() -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/{*path}", get(serve_static))
}

/// Run the UI server.
///
/// Starts an HTTP server to serve the embedded UI assets.
///
/// # Arguments
///
/// * `config` - UI server configuration
///
/// # Errors
///
/// Returns an error if the server fails to bind or run.
pub async fn run_ui_server(config: UiServerConfig) -> Result<(), std::io::Error> {
    if !config.enabled {
        tracing::info!("UI server is disabled");
        return Ok(());
    }

    let app = create_ui_router();

    let addr: SocketAddr = config
        .address()
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    tracing::info!("UI server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_server_config_default() {
        let config = UiServerConfig::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.bind_address, "127.0.0.1");
        assert!(config.enabled);
    }

    #[test]
    fn test_ui_server_config_with_port() {
        let config = UiServerConfig::with_port(8080);
        assert_eq!(config.port, 8080);
        assert_eq!(config.bind_address, "127.0.0.1");
    }

    #[test]
    fn test_ui_server_config_address() {
        let config = UiServerConfig::default();
        assert_eq!(config.address(), "127.0.0.1:3000");
    }
}
