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
