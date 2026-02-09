//! JSON-RPC protocol.

use serde::{Deserialize, Serialize};

/// JSON-RPC request.
#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    /// JSON-RPC version.
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Request parameters.
    pub params: serde_json::Value,
    /// Request ID.
    pub id: Option<String>,
}

/// JSON-RPC response.
#[derive(Debug, Serialize)]
pub struct RpcResponse {
    /// JSON-RPC version.
    pub jsonrpc: String,
    /// Result (if success).
    pub result: Option<serde_json::Value>,
    /// Error (if failure).
    pub error: Option<RpcError>,
    /// Request ID.
    pub id: Option<String>,
}

/// JSON-RPC error.
#[derive(Debug, Serialize)]
pub struct RpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional data.
    pub data: Option<serde_json::Value>,
}

impl RpcResponse {
    /// Create a success response.
    #[must_use]
    pub fn success(id: Option<String>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response.
    #[must_use]
    pub fn error(id: Option<String>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}

// Standard JSON-RPC error codes
/// Parse error.
pub const PARSE_ERROR: i32 = -32700;
/// Invalid request.
pub const INVALID_REQUEST: i32 = -32600;
/// Method not found.
pub const METHOD_NOT_FOUND: i32 = -32601;
/// Invalid params.
pub const INVALID_PARAMS: i32 = -32602;
/// Internal error.
pub const INTERNAL_ERROR: i32 = -32603;

// Application-specific error codes (using -32000 to -32099 range)
/// Unauthorized (authentication required or failed).
pub const UNAUTHORIZED: i32 = -32001;
/// Forbidden (insufficient permissions).
pub const FORBIDDEN: i32 = -32002;
/// Resource not found.
pub const NOT_FOUND: i32 = -32003;
