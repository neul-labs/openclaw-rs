//! # OpenClaw Agents
//!
//! Agent runtime, workflow engine, and sandboxed execution.
//!
//! Patterns from m9m: workflow nodes, bubblewrap sandboxing.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod runtime;
pub mod sandbox;
pub mod tools;
pub mod workflow;

pub use runtime::{AgentRuntime, AgentContext};
pub use sandbox::{SandboxConfig, SandboxLevel, execute_sandboxed, SandboxOutput};
pub use tools::ToolRegistry;
pub use workflow::{Workflow, WorkflowEngine, WorkflowNode};
