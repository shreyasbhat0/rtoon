pub mod parser;
pub mod scanner;

use crate::error::{ToonResult};
use crate::types::DecodeOptions;
use serde_json::Value;

pub fn decode(input: &str, options: &DecodeOptions) -> ToonResult<Value> {
    let mut parser = parser::Parser::new(input, options.clone());
    parser.parse()
}

pub fn decode_default(input: &str) -> ToonResult<Value> {
    decode(input, &DecodeOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_decode_null() {
        assert_eq!(decode_default("null").unwrap(), json!(null));
    }

    #[test]
    fn test_decode_bool() {
        assert_eq!(decode_default("true").unwrap(), json!(true));
        assert_eq!(decode_default("false").unwrap(), json!(false));
    }

    #[test]
    fn test_decode_number() {
        assert_eq!(decode_default("42").unwrap(), json!(42));
        assert_eq!(decode_default("3.14").unwrap(), json!(3.14));
        assert_eq!(decode_default("-5").unwrap(), json!(-5));
    }

    #[test]
    fn test_decode_string() {
        assert_eq!(decode_default("hello").unwrap(), json!("hello"));
        assert_eq!(
            decode_default("\"hello world\"").unwrap(),
            json!("hello world")
        );
    }

    #[test]
    fn test_decode_simple_object() {
        let input = "name: Alice\nage: 30";
        let result = decode_default(input).unwrap();
        assert_eq!(result["name"], json!("Alice"));
        assert_eq!(result["age"], json!(30));
    }

    #[test]
    fn test_decode_primitive_array() {
        let input = "tags[3]: reading,gaming,coding";
        let result = decode_default(input).unwrap();
        assert_eq!(
            result["tags"],
            json!(["reading", "gaming", "coding"])
        );
    }

    #[test]
    fn test_decode_tabular_array() {
        let input = "users[2]{id,name,role}:\n  1,Alice,admin\n  2,Bob,user";
        let result = decode_default(input).unwrap();
        assert_eq!(
            result["users"],
            json!([
                {"id": 1, "name": "Alice", "role": "admin"},
                {"id": 2, "name": "Bob", "role": "user"}
            ])
        );
    }

    #[test]
    fn test_decode_empty_array() {
        let input = "items[0]:";
        let result = decode_default(input).unwrap();
        assert_eq!(result["items"], json!([]));
    }

    #[test]
    fn test_decode_quoted_strings() {
        let input = "tags[3]: \"true\",\"42\",\"-3.14\"";
        let result = decode_default(input).unwrap();
        assert_eq!(result["tags"], json!(["true", "42", "-3.14"]));
    }
}