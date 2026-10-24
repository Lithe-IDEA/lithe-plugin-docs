//! Official SDK for Lithe plugin backends written in Rust.
//!
//! A plugin backend is the one long-lived process a plugin declares under
//! `backend` in `plugin.json`; Lithe starts it when one of the plugin's own
//! web pages first calls a function. It speaks the LSP base protocol on
//! stdio and implements, from
//! [plugin contract v1](https://github.com/Lithe-IDEA/lithe-plugin-docs/blob/main/contract/v1/README.md):
//!
//! - **`litheFunctions` 1 and 2**: plugin functions with progress,
//!   cancellation, and registration at runtime;
//! - **host services** of version 2: the plugin's own
//!   [`Credentials`] in the platform's secret store.
//!
//! The SDK owns the wire format, negotiation, cancellation, and error
//! mapping; plugin code supplies handlers. Each call runs on a thread of its
//! own, so handlers may block. A handler returns `Ok(value)` with a JSON
//! result, a [`PluginError`] the plugin's UI can tell apart by its code, or
//! any other error, which reaches the host only as a generic failure while
//! its details go to stderr. A panic ends only its call.
//!
//! See [`Backend`] for a minimal backend.

mod backend;
mod error;
mod framing;
mod protocol;
mod stdio;

pub use backend::{Backend, BackendHandle, Cancellation, Context, Credentials, Initialize};
pub use error::{sdk_codes, Cancelled, FunctionError, PluginError, SdkError};
pub use protocol::{is_credential_key, is_function_id, is_plugin_error_code};
pub use serde_json::Value;
