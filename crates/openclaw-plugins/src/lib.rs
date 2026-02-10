//! # OpenClaw Plugins
//!
//! Plugin system and FFI bridge for TypeScript, WASM, and native plugins.

#![warn(missing_docs)]

/// Plugin API traits and types.
pub mod api;
/// TypeScript plugin bridge.
pub mod bridge;
/// Native plugin FFI.
pub mod native;
/// Plugin registry.
pub mod registry;
/// WASM plugin runtime.
pub mod wasm;

pub use api::{Plugin, PluginApi, PluginError, PluginHook};
pub use bridge::{PluginInfo, SkillEntry, SkillManifest, TsPluginBridge, discover_plugins};
pub use native::{NativePlugin, NativePluginInfo, NativePluginManager, discover_native_plugins};
pub use registry::PluginRegistry;
pub use wasm::{WasmPlugin, WasmPluginManager, WasmPluginMetadata};
