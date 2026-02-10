//! IPC message types.

use serde::{Deserialize, Serialize};

/// IPC message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Message ID for request/response correlation.
    pub id: String,
    /// Message payload.
    pub payload: IpcPayload,
}

/// IPC message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcPayload {
    /// Request message.
    Request(IpcRequest),
    /// Response message.
    Response(IpcResponse),
    /// Event notification.
    Event(IpcEvent),
}

/// IPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    /// Method name.
    pub method: String,
    /// Request parameters.
    pub params: serde_json::Value,
}

/// IPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// Whether the request succeeded.
    pub success: bool,
    /// Result data (if success).
    pub result: Option<serde_json::Value>,
    /// Error message (if failure).
    pub error: Option<String>,
}

/// IPC event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEvent {
    /// Event type.
    pub event_type: String,
    /// Event data.
    pub data: serde_json::Value,
}

impl IpcMessage {
    /// Create a new request message.
    #[must_use]
    pub fn request(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            id: uuid_v4(),
            payload: IpcPayload::Request(IpcRequest {
                method: method.into(),
                params,
            }),
        }
    }

    /// Create a success response.
    #[must_use]
    pub fn success(id: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            payload: IpcPayload::Response(IpcResponse {
                success: true,
                result: Some(result),
                error: None,
            }),
        }
    }

    /// Create an error response.
    #[must_use]
    pub fn error(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            payload: IpcPayload::Response(IpcResponse {
                success: false,
                result: None,
                error: Some(error.into()),
            }),
        }
    }

    /// Create an event message.
    #[must_use]
    pub fn event(event_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            id: uuid_v4(),
            payload: IpcPayload::Event(IpcEvent {
                event_type: event_type.into(),
                data,
            }),
        }
    }
}

/// Generate a simple UUID v4.
fn uuid_v4() -> String {
    let bytes: [u8; 16] = rand::random();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        (bytes[6] & 0x0f) | 0x40,
        bytes[7],
        (bytes[8] & 0x3f) | 0x80,
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_message() {
        let msg = IpcMessage::request("test.method", serde_json::json!({"key": "value"}));
        assert!(!msg.id.is_empty());
        if let IpcPayload::Request(req) = msg.payload {
            assert_eq!(req.method, "test.method");
        } else {
            panic!("Expected request payload");
        }
    }

    #[test]
    fn test_success_response() {
        let msg = IpcMessage::success("123", serde_json::json!({"result": true}));
        if let IpcPayload::Response(resp) = msg.payload {
            assert!(resp.success);
            assert!(resp.result.is_some());
        } else {
            panic!("Expected response payload");
        }
    }
}
