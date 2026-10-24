//! Wire constants and validation shared with Lithe's Rust Core
//! (`rust/lithe-core/src/lsp/interface/plugin_function.rs` in the Lithe
//! repository). The SDK checks the same rules locally so a plugin author sees
//! a mistake when declaring a function instead of as a refused session.

/// Plugin-function protocol versions this SDK answers, newest first.
/// Version 2 keeps every rule of version 1 and adds host services.
pub const FUNCTIONS_VERSIONS: [u64; 2] = [2, 1];
/// First plugin-function protocol version with host services.
pub const HOST_SERVICES_VERSION: u64 = 2;

pub(crate) mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const SHUTDOWN: &str = "shutdown";
    pub const EXIT: &str = "exit";
    pub const CANCEL_REQUEST: &str = "$/cancelRequest";
    pub const FUNCTION_INVOKE: &str = "lithe/function/invoke";
    pub const FUNCTION_REGISTER: &str = "lithe/function/register";
    pub const FUNCTION_UNREGISTER: &str = "lithe/function/unregister";
    pub const FUNCTION_PROGRESS: &str = "lithe/function/progress";
    pub const CREDENTIALS_GET: &str = "lithe/host/credentials/get";
    pub const CREDENTIALS_SET: &str = "lithe/host/credentials/set";
    pub const CREDENTIALS_DELETE: &str = "lithe/host/credentials/delete";
}

/// JSON-RPC and LSP error codes the SDK answers with.
pub(crate) mod error_codes {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
    pub const SERVER_NOT_INITIALIZED: i64 = -32002;
    pub const REQUEST_FAILED: i64 = -32000;
    pub const REQUEST_CANCELLED: i64 = -32800;
}

pub(crate) const MAX_FUNCTIONS: usize = 256;
const MAX_FUNCTION_ID_BYTES: usize = 128;
const MAX_PLUGIN_ERROR_CODE_BYTES: usize = 64;
pub(crate) const MAX_MESSAGE_BYTES: usize = 1024;
pub(crate) const MAX_RESULT_BYTES: usize = 1 << 20;
const MAX_CREDENTIAL_KEY_BYTES: usize = 128;
pub(crate) const MAX_CREDENTIAL_SECRET_BYTES: usize = 16 * 1024;

/// Dot-separated lowerCamelCase segments, for example `snapshot.refresh`.
pub fn is_function_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FUNCTION_ID_BYTES
        && value.split('.').all(|segment| {
            let mut characters = segment.chars();
            characters
                .next()
                .is_some_and(|first| first.is_ascii_lowercase())
                && characters.all(|character| character.is_ascii_alphanumeric())
        })
}

/// The plugin error code alphabet Core forwards as `pluginErrorCode`.
pub fn is_plugin_error_code(value: &str) -> bool {
    let mut characters = value.chars();
    value.len() <= MAX_PLUGIN_ERROR_CODE_BYTES
        && characters
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic())
        && characters
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
}

/// A credential key: 1–128 ASCII letters, digits, `.`, `_`, `:`, or `-`.
pub fn is_credential_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CREDENTIAL_KEY_BYTES
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._:-".contains(character))
}

/// Cuts `value` to `limit` UTF-8 bytes without splitting a character.
pub(crate) fn bounded_text(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        return value.to_string();
    }
    let mut end = limit;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_ids_are_dot_separated_lower_camel_case() {
        assert!(is_function_id("snapshot.refresh"));
        assert!(is_function_id("query.runAll2"));
        for invalid in [
            "",
            "Snapshot",
            "snapshot.",
            ".a",
            "a..b",
            "a-b",
            "a.B",
            "1a",
        ] {
            assert!(!is_function_id(invalid), "{invalid}");
        }
        assert!(!is_function_id(&"a".repeat(129)));
    }

    #[test]
    fn plugin_error_codes_and_credential_keys_use_their_alphabets() {
        assert!(is_plugin_error_code("db.connection-failed_2"));
        assert!(!is_plugin_error_code("2db"));
        assert!(!is_plugin_error_code(&"a".repeat(65)));
        assert!(is_credential_key("connection:prod.password"));
        assert!(!is_credential_key(""));
        assert!(!is_credential_key("has space"));
        assert!(!is_credential_key(&"k".repeat(129)));
    }

    #[test]
    fn bounded_text_never_splits_a_character() {
        assert_eq!(bounded_text("abc", 8), "abc");
        assert_eq!(bounded_text("aé", 2), "a");
        assert_eq!(bounded_text("日本語", 7), "日本");
    }
}
