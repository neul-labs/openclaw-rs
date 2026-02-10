//! UI static file server with embedded assets.
//!
//! Serves the Vue 3 dashboard UI as embedded static files.
//! Assets are embedded at compile time using rust-embed when the `ui` feature is enabled.

use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use std::net::SocketAddr;

#[cfg(feature = "ui")]
use axum::{extract::Path, http::header};

#[cfg(feature = "ui")]
use rust_embed::Embed;

/// Embedded UI assets from the Vue build output.
///
/// Assets are embedded from `../openclaw-ui/dist` at compile time.
/// The `compression` feature compresses assets for smaller binary size.
///
/// This is only available when the `ui` feature is enabled.
#[cfg(feature = "ui")]
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

// ============================================================================
// UI feature enabled - serve embedded assets
// ============================================================================

#[cfg(feature = "ui")]
async fn serve_index() -> Response {
    serve_file("index.html").await
}

#[cfg(feature = "ui")]
async fn serve_static(Path(path): Path<String>) -> Response {
    serve_file(&path).await
}

#[cfg(feature = "ui")]
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

// ============================================================================
// UI feature disabled - return "not available" message
// ============================================================================

#[cfg(not(feature = "ui"))]
async fn serve_ui_not_available() -> Response {
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/html")],
        r#"<!DOCTYPE html>
<html>
<head>
    <title>OpenClaw UI</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 600px; margin: 100px auto; padding: 20px; }
        h1 { color: #333; }
        code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>OpenClaw UI Not Available</h1>
    <p>The web dashboard UI is not included in this build.</p>
    <p>To enable the UI, rebuild with the <code>ui</code> feature:</p>
    <pre><code>cargo build --features ui</code></pre>
    <p>The API gateway is running and accessible at the configured endpoints.</p>
</body>
</html>"#,
    )
        .into_response()
}

/// Create the UI router.
///
/// Sets up routes for serving static files with SPA fallback.
/// When the `ui` feature is disabled, returns a "not available" message.
#[cfg(feature = "ui")]
pub fn create_ui_router() -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/{*path}", get(serve_static))
}

/// Create the UI router (stub when UI feature is disabled).
#[cfg(not(feature = "ui"))]
pub fn create_ui_router() -> Router {
    Router::new()
        .route("/", get(serve_ui_not_available))
        .fallback(get(serve_ui_not_available))
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

    #[cfg(feature = "ui")]
    tracing::info!("UI server listening on http://{}", addr);

    #[cfg(not(feature = "ui"))]
    tracing::info!(
        "UI server listening on http://{} (UI feature not enabled)",
        addr
    );

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
