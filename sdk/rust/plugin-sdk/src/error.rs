//! The three ways a call can fail, and the JSON-RPC error each becomes.

use crate::protocol::{bounded_text, error_codes, is_plugin_error_code, MAX_MESSAGE_BYTES};
use serde_json::{json, Value};
use std::fmt;

/// A failure a handler reports on purpose. Core ends the call with the stable
/// `plugin.function.failed` code and forwards `code` as `pluginErrorCode`, so
/// the plugin's own UI can tell its failures apart. Lithe never branches on
/// it. The message is shown to the user as written, so it must not contain
/// secrets, stack traces, or machine paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginError {
    code: String,
    message: String,
}

impl PluginError {
    /// `code` is an identifier of at most 64 bytes, such as
    /// `db.connectionFailed`; Core keeps the first 1024 bytes of `message`.
    ///
    /// # Panics
    ///
    /// When `code` does not start with a letter, uses characters other than
    /// letters, digits, `.`, `_`, and `-`, or is longer than 64 bytes. In a
    /// handler, that panic ends the call as an internal error.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        let code = code.into();
        assert!(
            is_plugin_error_code(&code),
            "Plugin error code {code:?} must start with a letter, use letters, digits, \".\", \"_\" or \"-\", and be at most 64 bytes."
        );
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PluginError {}

/// A handler that noticed its call was cancelled; see
/// [`Cancellation::check`](crate::Cancellation::check).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cancelled;

impl fmt::Display for Cancelled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("The call was cancelled.")
    }
}

impl std::error::Error for Cancelled {}

/// An SDK usage error, or a host request that failed. `code` is the host's
/// `error.data.code` (such as `plugin.host.permissionDenied`) or one of the
/// SDK's own `sdk.*` codes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdkError {
    code: String,
    message: String,
    rpc_code: Option<i64>,
}

/// SDK error codes a backend may want to tell apart.
pub mod sdk_codes {
    /// The call that made the host request was cancelled first.
    pub const CALL_CANCELLED: &str = "sdk.call.cancelled";
    /// The session ended before the host answered.
    pub const SESSION_CLOSED: &str = "sdk.session.closed";
    /// The session did not negotiate the protocol version that offers this.
    pub const NOT_NEGOTIATED: &str = "sdk.protocol.notNegotiated";
}

impl SdkError {
    pub(crate) fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            rpc_code: None,
        }
    }

    pub(crate) fn from_response(method: &str, error: &Value) -> Self {
        Self {
            code: error["data"]["code"]
                .as_str()
                .unwrap_or("sdk.host.error")
                .to_string(),
            message: error["message"]
                .as_str()
                .map_or_else(|| format!("{method} failed."), str::to_string),
            rpc_code: error["code"].as_i64(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// The JSON-RPC error code of a host response, such as `-32601`.
    pub fn rpc_code(&self) -> Option<i64> {
        self.rpc_code
    }
}

impl fmt::Display for SdkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SdkError {}

/// How a function handler fails. `?` converts every error type a handler
/// typically meets: a [`PluginError`] keeps its code, [`Cancelled`] (and a
/// host request cut short by cancellation) answers as cancelled, and anything
/// else becomes a generic internal error whose details only reach stderr.
#[derive(Debug)]
pub enum FunctionError {
    Plugin(PluginError),
    Cancelled,
    /// Logged to stderr; the host sees only that the backend failed.
    Internal(String),
}

impl FunctionError {
    pub fn internal(details: impl fmt::Display) -> Self {
        Self::Internal(details.to_string())
    }
}

impl fmt::Display for FunctionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plugin(error) => error.fmt(formatter),
            Self::Cancelled => Cancelled.fmt(formatter),
            Self::Internal(details) => formatter.write_str(details),
        }
    }
}

impl From<PluginError> for FunctionError {
    fn from(error: PluginError) -> Self {
        Self::Plugin(error)
    }
}

impl From<Cancelled> for FunctionError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

impl From<SdkError> for FunctionError {
    fn from(error: SdkError) -> Self {
        if error.code == sdk_codes::CALL_CANCELLED {
            Self::Cancelled
        } else {
            Self::Internal(error.to_string())
        }
    }
}

impl From<std::io::Error> for FunctionError {
    fn from(error: std::io::Error) -> Self {
        Self::Internal(error.to_string())
    }
}

impl From<serde_json::Error> for FunctionError {
    fn from(error: serde_json::Error) -> Self {
        Self::Internal(error.to_string())
    }
}

pub(crate) fn cancelled_response() -> Value {
    json!({ "code": error_codes::REQUEST_CANCELLED, "message": "The request was cancelled." })
}

/// The JSON-RPC error object for a handler failure. Only a [`PluginError`]
/// carries a plugin code; anything else is a generic internal error, so
/// stack traces and machine paths never reach the host's UI.
pub(crate) fn response_error(error: &FunctionError) -> Value {
    match error {
        FunctionError::Cancelled => cancelled_response(),
        FunctionError::Plugin(error) => json!({
            "code": error_codes::REQUEST_FAILED,
            "message": bounded_text(&error.message, MAX_MESSAGE_BYTES),
            "data": { "code": error.code },
        }),
        FunctionError::Internal(_) => internal_response(),
    }
}

pub(crate) fn internal_response() -> Value {
    json!({ "code": error_codes::INTERNAL_ERROR, "message": "The plugin backend failed unexpectedly." })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plugin_errors_reach_the_host_with_their_message_and_code() {
        let plugin = response_error(&PluginError::new("db.locked", "é".repeat(600)).into());
        assert_eq!(plugin["code"], -32000);
        assert_eq!(plugin["data"]["code"], "db.locked");
        assert_eq!(plugin["message"].as_str().unwrap().len(), 1024);
        let internal = response_error(&FunctionError::internal("/home/me/secret: No such file"));
        assert_eq!(internal, internal_response());
        assert_eq!(response_error(&Cancelled.into())["code"], -32800);
        let cut_short: FunctionError = SdkError::new(sdk_codes::CALL_CANCELLED, "x").into();
        assert!(matches!(cut_short, FunctionError::Cancelled));
    }

    #[test]
    #[should_panic(expected = "must start with a letter")]
    fn an_invalid_plugin_error_code_is_a_programming_error() {
        let _ = PluginError::new("1bad", "message");
    }
}
