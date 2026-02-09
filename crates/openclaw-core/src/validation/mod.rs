//! Input validation and sanitization.
//!
//! Defense-in-depth: validate all external inputs before processing.

use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

/// Validation error types.
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Input exceeds maximum allowed length.
    #[error("Input exceeds maximum length ({max} bytes, got {actual})")]
    TooLong {
        /// Maximum allowed length.
        max: usize,
        /// Actual input length.
        actual: usize,
    },

    /// Invalid UTF-8 encoding.
    #[error("Invalid UTF-8 encoding")]
    InvalidUtf8,

    /// Disallowed characters in input.
    #[error("Disallowed characters in input")]
    DisallowedChars,

    /// Input failed schema validation.
    #[error("Input failed schema validation: {0}")]
    SchemaViolation(String),

    /// JSON parsing error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Size limits per input type.
pub mod limits {
    /// Maximum message content length (64KB).
    pub const MAX_MESSAGE_LENGTH: usize = 64 * 1024;

    /// Maximum tool parameters size (1MB).
    pub const MAX_TOOL_PARAMS_SIZE: usize = 1024 * 1024;

    /// Maximum skill file size (256KB).
    pub const MAX_SKILL_FILE_SIZE: usize = 256 * 1024;

    /// Maximum config file size (1MB).
    pub const MAX_CONFIG_FILE_SIZE: usize = 1024 * 1024;

    /// Maximum attachment size (50MB).
    pub const MAX_ATTACHMENT_SIZE: usize = 50 * 1024 * 1024;

    /// Maximum JSON nesting depth.
    pub const MAX_JSON_DEPTH: usize = 32;
}

/// Validate and sanitize message content from channels.
///
/// Performs:
/// 1. Length check (prevent memory exhaustion)
/// 2. Strip null bytes and control chars (except newlines/tabs)
/// 3. Unicode normalization (NFKC - prevent homograph attacks)
///
/// # Errors
///
/// Returns `ValidationError::TooLong` if input exceeds `max_len`.
pub fn validate_message_content(input: &str, max_len: usize) -> Result<String, ValidationError> {
    // 1. Length check (prevent memory exhaustion)
    if input.len() > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: input.len(),
        });
    }

    // 2. Strip null bytes and control chars (except newlines/tabs)
    let sanitized: String = input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t' || *c == '\r')
        .collect();

    // 3. Normalize unicode (NFKC - prevent homograph attacks in allowlists)
    let normalized: String = sanitized.nfkc().collect();

    Ok(normalized)
}

/// Validate tool parameters against a JSON schema.
///
/// # Errors
///
/// Returns `ValidationError::SchemaViolation` if validation fails.
pub fn validate_tool_params(
    params: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<(), ValidationError> {
    // Check size limit first
    let size = serde_json::to_string(params)?.len();
    if size > limits::MAX_TOOL_PARAMS_SIZE {
        return Err(ValidationError::TooLong {
            max: limits::MAX_TOOL_PARAMS_SIZE,
            actual: size,
        });
    }

    // Check JSON depth
    check_json_depth(params, 0, limits::MAX_JSON_DEPTH)?;

    // JSON Schema validation would go here
    // For now, we do basic structural validation
    validate_json_structure(params, schema)?;

    Ok(())
}

/// Check JSON nesting depth to prevent stack overflow.
fn check_json_depth(
    value: &serde_json::Value,
    depth: usize,
    max: usize,
) -> Result<(), ValidationError> {
    if depth > max {
        return Err(ValidationError::SchemaViolation(format!(
            "JSON nesting depth exceeds maximum ({max})"
        )));
    }

    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                check_json_depth(item, depth + 1, max)?;
            }
        }
        serde_json::Value::Object(obj) => {
            for (_, item) in obj {
                check_json_depth(item, depth + 1, max)?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Basic JSON structure validation against schema.
fn validate_json_structure(
    params: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<(), ValidationError> {
    let schema_type = schema.get("type").and_then(|t| t.as_str());

    match schema_type {
        Some("object") => {
            if !params.is_object() {
                return Err(ValidationError::SchemaViolation(
                    "Expected object".to_string(),
                ));
            }

            // Check required fields
            if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
                let obj = params.as_object().unwrap();
                for req in required {
                    if let Some(field) = req.as_str() {
                        if !obj.contains_key(field) {
                            return Err(ValidationError::SchemaViolation(format!(
                                "Missing required field: {field}"
                            )));
                        }
                    }
                }
            }
        }
        Some("array") => {
            if !params.is_array() {
                return Err(ValidationError::SchemaViolation(
                    "Expected array".to_string(),
                ));
            }
        }
        Some("string") => {
            if !params.is_string() {
                return Err(ValidationError::SchemaViolation(
                    "Expected string".to_string(),
                ));
            }
        }
        Some("number") | Some("integer") => {
            if !params.is_number() {
                return Err(ValidationError::SchemaViolation(
                    "Expected number".to_string(),
                ));
            }
        }
        Some("boolean") => {
            if !params.is_boolean() {
                return Err(ValidationError::SchemaViolation(
                    "Expected boolean".to_string(),
                ));
            }
        }
        _ => {}
    }

    Ok(())
}

/// Validate a file path to prevent path traversal attacks.
///
/// # Errors
///
/// Returns error if path contains traversal sequences.
pub fn validate_path(path: &str) -> Result<(), ValidationError> {
    if path.contains("..") || path.contains('\0') {
        return Err(ValidationError::DisallowedChars);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_message_content() {
        // Normal content
        let result = validate_message_content("Hello, world!", 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!");

        // With control chars
        let result = validate_message_content("Hello\x00World", 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HelloWorld");

        // Preserves newlines
        let result = validate_message_content("Line1\nLine2", 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Line1\nLine2");

        // Too long
        let result = validate_message_content("x".repeat(200).as_str(), 100);
        assert!(matches!(result, Err(ValidationError::TooLong { .. })));
    }

    #[test]
    fn test_unicode_normalization() {
        // NFKC normalization
        let result = validate_message_content("ﬁ", 100); // fi ligature
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fi");
    }

    #[test]
    fn test_validate_path() {
        assert!(validate_path("/home/user/file.txt").is_ok());
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("/home/user/\0file").is_err());
    }

    #[test]
    fn test_json_depth() {
        let shallow = serde_json::json!({"a": {"b": "c"}});
        assert!(check_json_depth(&shallow, 0, 10).is_ok());

        // Create deeply nested JSON
        let mut deep = serde_json::json!("leaf");
        for _ in 0..50 {
            deep = serde_json::json!({"nested": deep});
        }
        assert!(check_json_depth(&deep, 0, 32).is_err());
    }
}
