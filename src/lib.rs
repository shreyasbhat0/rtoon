//! # RToon
//!
//! A Rust implementation of TOON (Token-Oriented Object Notation), a compact
//! format for structured data optimized for LLM token efficiency.
//!
//! # Examples
//!
//! ```
//! use rtoon::{encode_default, decode_default};
//! use serde_json::json;
//!
//! let data = json!({"name": "Alice", "age": 30});
//! let encoded = encode_default(&data)?;
//! let decoded = decode_default(&encoded)?;
//! # Ok::<(), rtoon::ToonError>(())
//! ```

pub mod constants;
pub mod decode;
pub mod encode;
pub mod error;
pub mod types;
pub mod utils;

pub use decode::{
    decode,
    decode_default,
    decode_no_coerce,
    decode_no_coerce_with_options,
    decode_strict,
    decode_strict_with_options,
};
pub use encode::{
    encode,
    encode_array,
    encode_default,
    encode_object,
};
pub use error::{
    ToonError,
    ToonResult,
};
use serde::{Deserialize, Serialize};
pub use types::{
    DecodeOptions,
    Delimiter,
    EncodeOptions,
};
pub use utils::{
    literal::{
        is_keyword,
        is_literal_like,
    },
    normalize,
    string::{
        escape_string,
        is_valid_unquoted_key,
        needs_quoting,
    },
};


/// Serialize any Rust type that implements `Serialize` to TOON format.
///
/// This function converts the value to JSON first, then encodes it to TOON.
/// You can optionally provide custom encoding options to control the output format.
///
/// # Arguments
///
/// * `value` - The value to serialize (must implement `Serialize`)
/// * `options` - Optional encoding options. If `None`, uses default options.
///
/// # Examples
///
/// ## Basic usage (no options)
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// let user = User {
///     name: "Alice".to_string(),
///     age: 30,
/// };
///
/// // Use default options
/// let toon = rtoon::to_toon(&user, None)?;
/// assert!(toon.contains("name: Alice"));
/// # Ok::<(), rtoon::ToonError>(())
/// ```
///
/// ## With custom options
///
/// ```
/// use rtoon::{EncodeOptions, Delimiter};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Data {
///     tags: Vec<String>,
/// }
///
/// let data = Data {
///     tags: vec!["rust".to_string(), "toon".to_string()],
/// };
///
/// let options = EncodeOptions::new()
///     .with_delimiter(Delimiter::Pipe)
///     .with_length_marker('#');
///
/// let toon = rtoon::to_toon(&data, Some(&options))?;
/// assert!(toon.contains("|"));
/// assert!(toon.contains("[#"));
/// # Ok::<(), rtoon::ToonError>(())
/// ```
pub fn to_toon<T: Serialize>(
    value: &T,
    options: Option<&EncodeOptions>,
) -> ToonResult<String> {
    let json_value = serde_json::to_value(value)
        .map_err(|e| ToonError::InvalidInput(format!("Serialization error: {}", e)))?;
    
    match options {
        Some(opts) => encode(&json_value, opts),
        None => encode_default(&json_value),
    }
}

/// Deserialize TOON format directly to any Rust type that implements `Deserialize`.
///
/// This function decodes TOON to JSON first, then deserializes it to the target type.
/// You can optionally provide custom decoding options to control parsing behavior.
///
/// # Arguments
///
/// * `s` - The TOON string to deserialize
/// * `options` - Optional decoding options. If `None`, uses default options (strict mode enabled).
///
/// # Examples
///
/// ## Basic usage (no options)
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// let toon = "name: Alice\nage: 30";
/// 
/// // Use default options (strict mode)
/// let user: User = rtoon::from_toon(toon, None)?;
/// assert_eq!(user.name, "Alice");
/// assert_eq!(user.age, 30);
/// # Ok::<(), rtoon::ToonError>(())
/// ```
///
/// ## With custom options
///
/// ```
/// use rtoon::DecodeOptions;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct Data {
///     tags: Vec<String>,
/// }
///
/// let toon = "tags[3]: reading,gaming,coding";
/// 
/// let options = DecodeOptions::new()
///     .with_strict(false)
///     .with_coerce_types(true);
///
/// let data: Data = rtoon::from_toon(toon, Some(&options))?;
/// assert_eq!(data.tags, vec!["reading", "gaming", "coding"]);
/// # Ok::<(), rtoon::ToonError>(())
/// ```
///
/// ## Round-trip conversion
///
/// ```
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Config {
///     host: String,
///     port: u16,
/// }
///
/// let original = Config {
///     host: "localhost".to_string(),
///     port: 8080,
/// };
///
/// let toon = rtoon::to_toon(&original, None)?;
/// let decoded: Config = rtoon::from_toon(&toon, None)?;
/// assert_eq!(original, decoded);
/// # Ok::<(), rtoon::ToonError>(())
/// ```
pub fn from_toon<T: for<'de> Deserialize<'de>>(
    s: &str,
    options: Option<&DecodeOptions>,
) -> ToonResult<T> {
    let json_value = match options {
        Some(opts) => decode(s, opts)?,
        None => decode_default(s)?,
    };
    
    serde_json::from_value(json_value)
        .map_err(|e| ToonError::InvalidInput(format!("Deserialization error: {}", e)))
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decode::decode_strict;

    #[test]
    fn test_round_trip_simple() {
        let original = json!({"name": "Alice", "age": 30});
        let encoded = encode_default(&original).unwrap();
        let decoded = decode_default(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_round_trip_array() {
        let original = json!({"tags": ["reading", "gaming", "coding"]});
        let encoded = encode_default(&original).unwrap();
        let decoded = decode_default(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_round_trip_tabular() {
        let original = json!({
            "users": [
                {"id": 1, "name": "Alice", "role": "admin"},
                {"id": 2, "name": "Bob", "role": "user"}
            ]
        });
        let encoded = encode_default(&original).unwrap();
        let decoded = decode_default(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_custom_delimiter() {
        let original = json!({"tags": ["a", "b", "c"]});
        let opts = EncodeOptions::new().with_delimiter(Delimiter::Pipe);
        let encoded = encode(&original, &opts).unwrap();
        assert!(encoded.contains("|"));

        let decoded = decode_default(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_length_marker() {
        let original = json!({"tags": ["a", "b", "c"]});
        let opts = EncodeOptions::new().with_length_marker('#');
        let encoded = encode(&original, &opts).unwrap();
        assert!(encoded.contains("[#3]"));

        let decoded = decode_default(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_decode_strict_helper() {
        let input = "items[2]: a,b";
        assert!(decode_strict(input).is_ok());

        let input = "items[3]: a,b";
        assert!(decode_strict(input).is_err());
    }

    #[test]
    fn test_normalize_exported() {
        let value = json!(f64::NAN);
        let normalized = normalize(value);
        assert_eq!(normalized, json!(null));
    }

    #[test]
    fn test_utilities_exported() {
        assert!(is_keyword("null"));
        assert!(is_literal_like("true"));
        assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
        assert!(needs_quoting("true", Delimiter::Comma));
    }
}
