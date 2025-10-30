pub mod decode;
pub mod encode;
pub mod error;
pub mod utils;
pub mod types;

pub use decode::{decode, decode_default};
pub use encode::{encode, encode_default};
pub use error::{ToonResult, ToonError};
pub use utils::normalize;
pub use types::{DecodeOptions, Delimiter, EncodeOptions};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}