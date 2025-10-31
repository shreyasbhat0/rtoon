use serde_json::Value;

use crate::error::{
    ToonError,
    ToonResult,
};

/// Validate that nesting depth doesn't exceed the maximum.
pub fn validate_depth(depth: usize, max_depth: usize) -> ToonResult<()> {
    if depth > max_depth {
        return Err(ToonError::InvalidStructure(format!(
            "Maximum nesting depth of {} exceeded",
            max_depth
        )));
    }
    Ok(())
}

/// Validate that a field name is not empty.
pub fn validate_field_name(name: &str) -> ToonResult<()> {
    if name.is_empty() {
        return Err(ToonError::InvalidInput(
            "Field name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

/// Recursively validate a JSON value and all nested fields.
pub fn validate_value(value: &Value) -> ToonResult<()> {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj.iter() {
                validate_field_name(key)?;
                validate_value(val)?;
            }
        }
        Value::Array(arr) => {
            for val in arr.iter() {
                validate_value(val)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_validate_depth() {
        assert!(validate_depth(0, 10).is_ok());
        assert!(validate_depth(5, 10).is_ok());
        assert!(validate_depth(10, 10).is_ok());
        assert!(validate_depth(11, 10).is_err());
    }

    #[test]
    fn test_validate_field_name() {
        assert!(validate_field_name("name").is_ok());
        assert!(validate_field_name("user_id").is_ok());
        assert!(validate_field_name("").is_err());
    }

    #[test]
    fn test_validate_value() {
        assert!(validate_value(&json!(null)).is_ok());
        assert!(validate_value(&json!(123)).is_ok());
        assert!(validate_value(&json!("hello")).is_ok());
        assert!(validate_value(&json!({"name": "Alice"})).is_ok());
        assert!(validate_value(&json!([1, 2, 3])).is_ok());

        let bad_obj = json!({"": "value"});
        assert!(validate_value(&bad_obj).is_err());
    }
}
