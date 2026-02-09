//! # OpenClaw Plugins
//!
//! Plugin system and FFI bridge for TypeScript, WASM, and native plugins.

#![warn(missing_docs)]

/// Plugin API traits and types.
pub mod api;
/// Plugin registry.
pub mod registry;
/// TypeScript plugin bridge.
pub mod bridge;
/// WASM plugin runtime.
pub mod wasm;
/// Native plugin FFI.
pub mod native;

pub use api::{Plugin, PluginApi, PluginHook, PluginError};
pub use registry::PluginRegistry;
pub use bridge::{TsPluginBridge, SkillManifest, SkillEntry, PluginInfo, discover_plugins};
pub use wasm::{WasmPlugin, WasmPluginMetadata, WasmPluginManager};
pub use native::{NativePlugin, NativePluginInfo, NativePluginManager, discover_native_plugins};
