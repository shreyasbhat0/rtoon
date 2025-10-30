pub mod primitives;
pub mod writer;

use crate::error::{ToonResult, ToonError};
use crate::utils::normalize;
use crate::types::EncodeOptions;
use serde_json::Value;

pub fn encode(value: &Value, options: &EncodeOptions) -> ToonResult<String> {
    let normalized = normalize(value.clone());
    let mut writer = writer::Writer::new(options.clone());
    match &normalized {
        Value::Array(arr) => {
            encode_array(&mut writer, None, arr, 0)?;
        }
        Value::Object(obj) => {
            encode_object(&mut writer, obj, 0)?;
        }
        _ => {
            encode_primitive_value(&mut writer, &normalized)?;
        }
    }
    Ok(writer.finish())
}

pub fn encode_default(value: &Value) -> ToonResult<String> {
    encode(value, &EncodeOptions::default())
}

fn encode_array(writer: &mut writer::Writer, key: Option<&str>, arr: &[Value], depth: usize) -> ToonResult<()> {
    if arr.is_empty() {
        writer.write_empty_array_with_key(key)?;
        return Ok(());
    }

    if let Some(keys) = is_tabular_array(arr) {
        encode_tabular_array(writer, key, arr, &keys, depth)?;
    } else if is_primitive_array(arr) {
        encode_primitive_array(writer, key, arr, depth)?;
    } else {
        encode_nested_array(writer, key, arr, depth)?;
    }

    Ok(())
}

fn is_tabular_array(arr: &[Value]) -> Option<Vec<String>> {
    if arr.is_empty() {
        return None;
    }

    let first = arr.first()?;
    if !first.is_object() {
        return None;
    }

    let first_obj = first.as_object()?;
    let keys: Vec<String> = first_obj.keys().cloned().collect();

    for value in first_obj.values() {
        if !is_primitive(value) {
            return None;
        }
    }

    for val in arr.iter().skip(1) {
        if let Some(obj) = val.as_object() {
            let obj_keys: Vec<String> = obj.keys().cloned().collect();
            if keys != obj_keys {
                return None;
            }
            for value in obj.values() {
                if !is_primitive(value) {
                    return None;
                }
            }
        } else {
            return None;
        }
    }

    Some(keys)
}

fn is_primitive(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

fn is_primitive_array(arr: &[Value]) -> bool {
    arr.iter().all(is_primitive)
}

fn encode_primitive_array(writer: &mut writer::Writer, key: Option<&str>, arr: &[Value], depth: usize) -> ToonResult<()> {
    writer.write_array_header(key, arr.len(), None, depth)?;
    writer.write_str(" ")?;
    for (i, val) in arr.iter().enumerate() {
        if i > 0 {
            writer.write_delimiter()?;
        }
        encode_primitive_value(writer, val)?;
    }
    Ok(())
}

fn encode_primitive_value(writer: &mut writer::Writer, value: &Value) -> ToonResult<()> {
    match value {
        Value::Null => writer.write_str("null"),
        Value::Bool(b) => writer.write_str(&b.to_string()),
        Value::Number(n) => writer.write_str(&n.to_string()),
        Value::String(s) => {
            if writer.needs_quoting(s) {
                writer.write_quoted_string(s)
            } else {
                writer.write_str(s)
            }
        }
        _ => Err(ToonError::InvalidInput("Expected primitive value".to_string())),
    }
}

fn encode_tabular_array(
    writer: &mut writer::Writer,
    key: Option<&str>,
    arr: &[Value],
    keys: &[String],
    depth: usize,
) -> ToonResult<()> {
    writer.write_array_header(key, arr.len(), Some(keys), depth)?;
    writer.write_newline()?;
    for (row_index, obj_val) in arr.iter().enumerate() {
        if let Some(obj) = obj_val.as_object() {
            writer.write_indent(depth + 1)?;
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    writer.write_delimiter()?;
                }
                if let Some(val) = obj.get(key) {
                    encode_primitive_value(writer, val)?;
                } else {
                    writer.write_str("null")?;
                }
            }
            if row_index + 1 < arr.len() {
                writer.write_newline()?;
            }
        }
    }

    Ok(())
}

fn encode_nested_array(writer: &mut writer::Writer, key: Option<&str>, arr: &[Value], depth: usize) -> ToonResult<()> {
    writer.write_array_header(key, arr.len(), None, depth)?;
    writer.write_newline()?;

    for val in arr {
        writer.write_indent(depth + 1)?;
        writer.write_str("- ")?;

        match val {
            Value::Array(inner) if is_primitive_array(inner) => {
                encode_primitive_array(writer, None, inner, depth + 1)?;
                writer.write_newline()?;
            }
            Value::Object(obj) => {
                encode_object_as_list_item(writer, obj, depth + 1)?;
            }
            _ => {
                encode_primitive_value(writer, val)?;
                writer.write_newline()?;
            }
        }
    }

    Ok(())
}

fn encode_object_as_list_item(
    writer: &mut writer::Writer,
    obj: &serde_json::Map<String, Value>,
    depth: usize,
) -> ToonResult<()> {
    let keys: Vec<&String> = obj.keys().collect();
    
    if keys.is_empty() {
        writer.write_newline()?;
        return Ok(());
    }

    let first_key = keys[0];
    writer.write_str(first_key)?;
    writer.write_str(": ")?;
    
    if let Some(first_val) = obj.get(first_key) {
        match first_val {
            Value::Array(arr) => {
                encode_array(writer, None, arr, depth)?;
            }
            Value::Object(nested_obj) => {
                writer.write_newline()?;
                encode_object(writer, nested_obj, depth + 1)?;
            }
            _ => {
                encode_primitive_value(writer, first_val)?;
            }
        }
    }
    writer.write_newline()?;

    for key in keys.iter().skip(1) {
        writer.write_indent(depth)?;
        writer.write_str(key)?;
        writer.write_str(": ")?;
        
        if let Some(val) = obj.get(*key) {
            match val {
                Value::Array(arr) => {
                    encode_array(writer, None, arr, depth)?;
                    writer.write_newline()?;
                }
                Value::Object(nested_obj) => {
                    writer.write_newline()?;
                    encode_object(writer, nested_obj, depth + 1)?;
                }
                _ => {
                    encode_primitive_value(writer, val)?;
                    writer.write_newline()?;
                }
            }
        }
    }

    Ok(())
}

fn encode_object(writer: &mut writer::Writer, obj: &serde_json::Map<String, Value>, depth: usize) -> ToonResult<()> {
    for (i, (key, val)) in obj.iter().enumerate() {
        if i > 0 {
            writer.write_newline()?;
        }
        writer.write_indent(depth)?;
        match val {
            Value::Array(arr) => {
                encode_array(writer, Some(key), arr, depth)?;
            }
            Value::Object(nested) => {
                writer.write_key(key)?;
                writer.write_str(":")?;
                writer.write_newline()?;
                encode_object(writer, nested, depth + 1)?;
            }
            _ => {
                writer.write_key(key)?;
                writer.write_str(": ")?;
                encode_primitive_value(writer, val)?;
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::types::Delimiter;

    #[test]
    fn test_encode_simple_object() {
        let val = json!({"name": "Alice", "age": 30});
        let result = encode_default(&val).unwrap();
        assert!(result.contains("name: Alice"));
        assert!(result.contains("age: 30"));
    }

    #[test]
    fn test_encode_primitive_array() {
        let val = json!({"tags": ["reading", "gaming", "coding"]});
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "tags[3]: reading,gaming,coding");
    }

    #[test]
    fn test_encode_tabular_array() {
        let val = json!({
            "users": [
                {"id": 1, "name": "Alice", "role": "admin"},
                {"id": 2, "name": "Bob", "role": "user"}
            ]
        });
        let result = encode_default(&val).unwrap();
        assert!(result.contains("users[2]{id,name,role}:"));
        assert!(result.contains("1,Alice,admin"));
        assert!(result.contains("2,Bob,user"));
    }

    #[test]
    fn test_encode_empty_array() {
        let val = json!({"items": []});
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "items[0]:");
    }

    #[test]
    fn test_encode_with_pipe_delimiter() {
        let val = json!({"tags": ["a", "b", "c"]});
        let opts = EncodeOptions::new().with_delimiter(Delimiter::Pipe);
        let result = encode(&val, &opts).unwrap();
        assert_eq!(result, "tags[3|]: a|b|c");
    }

    #[test]
    fn test_encode_with_length_marker() {
        let val = json!({"tags": ["a", "b", "c"]});
        let opts = EncodeOptions::new().with_length_marker('#');
        let result = encode(&val, &opts).unwrap();
        assert_eq!(result, "tags[#3]: a,b,c");
    }

    #[test]
    fn test_encode_normalizes_special_values() {
        let val = json!({"value": null});
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "value: null");
    }

    #[test]
    fn test_encode_root_primitive_array() {
        let val = json!(["a", "b", "c"]);
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "[3]: a,b,c");
    }

    #[test]
    fn test_encode_root_tabular_array() {
        let val = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]);
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "[2]{id,name}:\n  1,Alice\n  2,Bob");
    }

    #[test]
    fn test_encode_root_primitive() {
        let val = json!(42);
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "42");

        let val = json!("hello");
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "hello");

        let val = json!(true);
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_encode_root_empty_array() {
        let val = json!([]);
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "[0]:");
    }

    #[test]
    fn test_encode_root_mixed_array() {
        let val = json!([1, "hello", true]);
        let result = encode_default(&val).unwrap();
        assert_eq!(result, "[3]: 1,hello,true");
    }

    #[test]
    fn test_encode_root_nested_array() {
        let val = json!([
            ["a", "b"],
            ["c", "d"]
        ]);
        let result = encode_default(&val).unwrap();
        assert!(result.contains("[2]:"));
        assert!(result.contains("- [2]: a,b"));
        assert!(result.contains("- [2]: c,d"));
    }
}