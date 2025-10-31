pub mod literal;
pub mod string;
pub mod validation;

pub use literal::{
    is_keyword,
    is_literal_like,
    is_numeric_like,
    is_structural_char,
};
use serde_json::{
    Map,
    Number,
    Value,
};
pub use string::{
    escape_string,
    is_valid_unquoted_key,
    needs_quoting,
    quote_string,
    unescape_string,
};

/// Context for determining when quoting is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotingContext {
    Key,
    Value,
    Header,
}

/// Normalize a JSON value (converts NaN/Infinity to null, -0 to 0).
pub fn normalize(value: Value) -> Value {
    match value {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.is_nan() || f.is_infinite() {
                    Value::Null
                } else if f == 0.0 && f.is_sign_negative() {
                    Value::Number(Number::from(0))
                } else {
                    Value::Number(n)
                }
            } else {
                Value::Number(n)
            }
        }
        Value::Object(obj) => {
            let normalized: Map<String, Value> =
                obj.into_iter().map(|(k, v)| (k, normalize(v))).collect();
            Value::Object(normalized)
        }
        Value::Array(arr) => {
            let normalized: Vec<Value> = arr.into_iter().map(normalize).collect();
            Value::Array(normalized)
        }
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_normalize_nan() {
        let value = json!(f64::NAN);
        let normalized = normalize(value);
        assert_eq!(normalized, json!(null));
    }

    #[test]
    fn test_normalize_infinity() {
        let value = json!(f64::INFINITY);
        let normalized = normalize(value);
        assert_eq!(normalized, json!(null));

        let value = json!(f64::NEG_INFINITY);
        let normalized = normalize(value);
        assert_eq!(normalized, json!(null));
    }

    #[test]
    fn test_normalize_negative_zero() {
        let value = json!(-0.0);
        let normalized = normalize(value);
        assert_eq!(normalized, json!(0));
    }

    #[test]
    fn test_normalize_nested() {
        let value = json!({
            "a": f64::NAN,
            "b": {
                "c": f64::INFINITY
            },
            "d": [1, f64::NAN, 3]
        });

        let normalized = normalize(value);
        assert_eq!(
            normalized,
            json!({
                "a": null,
                "b": {
                    "c": null
                },
                "d": [1, null, 3]
            })
        );
    }

    #[test]
    fn test_normalize_normal_values() {
        let value = json!({
            "name": "Alice",
            "age": 30,
            "score": 3.14
        });

        let normalized = normalize(value.clone());
        assert_eq!(normalized, value);
    }
}
