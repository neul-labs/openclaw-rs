//! # OpenClaw IPC
//!
//! IPC message types and nng transport for daemon communication.
//!
//! Uses the grite pattern: rkyv serialization over nng sockets.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod messages;
pub mod transport;

pub use messages::{IpcMessage, IpcRequest, IpcResponse};
pub use transport::IpcTransport;
