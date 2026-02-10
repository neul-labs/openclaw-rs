//! # `OpenClaw` Agents
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

pub use runtime::{AgentContext, AgentRuntime};
pub use sandbox::{SandboxConfig, SandboxLevel, SandboxOutput, execute_sandboxed};
pub use tools::ToolRegistry;
pub use workflow::{Workflow, WorkflowEngine, WorkflowNode};
