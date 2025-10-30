use crate::types::Delimiter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotingContext {
    Key,
    Value,
    Header,
}

pub fn is_literal_like(s: &str) -> bool {
    matches!(s, "true" | "false" | "null") || is_numeric_like(s)
}

fn is_numeric_like(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    if chars[i] == '-' {
        i += 1;
    }

    if i >= chars.len() {
        return false;
    }

    if !chars[i].is_ascii_digit() {
        return false;
    }

    if chars[i] == '0' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
        return false;
    }

    let numeric_chars = &chars[i..];
    let has_valid_chars = numeric_chars.iter().all(|c| {
        c.is_ascii_digit() || *c == '.' || *c == 'e' || *c == 'E' || *c == '+' || *c == '-'
    });

    if !has_valid_chars {
        return false;
    }

    if let Some(last) = chars.last() {
        if *last == '+' || *last == '-' {
            return false;
        }
    }

    true
}

pub fn needs_quoting(s: &str, context: QuotingContext, delimiter: Delimiter) -> bool {
    if s.is_empty() {
        return true;
    }

    if s != s.trim() {
        return true;
    }

    if is_literal_like(s) {
        return true;
    }

    if s.starts_with('[')
        || s.starts_with(']')
        || s.starts_with('{')
        || s.starts_with('}')
        || s.contains('[')
        || s.contains(']')
        || s.contains('{')
        || s.contains('}')
    {
        return true;
    }

    if s.contains(':') {
        return true;
    }

    if s.contains(delimiter.as_char()) {
        return true;
    }

    if s.chars()
        .any(|c| c.is_control() || c == '\\' || c == '"')
    {
        return true;
    }

    if context == QuotingContext::Value {
        if s == "-" {
            return true;
        }
        if s.starts_with("- ") {
            return true;
        }
    }

    false
}

pub fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 10);

    for c in s.chars() {
        match c {
            '"' => result.push_str(r#"\""#),
            '\\' => result.push_str(r"\\"),
            '\n' => result.push_str(r"\n"),
            '\r' => result.push_str(r"\r"),
            '\t' => result.push_str(r"\t"),
            _ => result.push(c),
        }
    }

    result
}

pub fn unescape_string(s: &str) -> Result<String, String> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some(other) => {
                    return Err(format!("Invalid escape sequence: \\{}", other));
                }
                None => {
                    return Err("Unterminated escape sequence".to_string());
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

pub fn format_quoted_string(s: &str) -> String {
    format!("\"{}\"", escape_string(s))
}

use serde_json::{Map as __Map, Number as __Number, Value as __Value};

pub fn normalize(value: __Value) -> __Value {
    match value {
        __Value::Null => __Value::Null,
        __Value::Bool(b) => __Value::Bool(b),
        __Value::Number(n) => normalize_number(n),
        __Value::String(s) => __Value::String(s),
        __Value::Array(arr) => normalize_array(arr),
        __Value::Object(obj) => normalize_object(obj),
    }
}

fn normalize_number(n: __Number) -> __Value {
    if let Some(f) = n.as_f64() {
        if f.is_nan() || f.is_infinite() {
            return __Value::Null;
        }
        if f == 0.0 && f.is_sign_negative() {
            return __Value::Number(__Number::from(0));
        }
    }
    __Value::Number(n)
}

fn normalize_array(arr: Vec<__Value>) -> __Value {
    __Value::Array(arr.into_iter().map(normalize).collect())
}

fn normalize_object(obj: __Map<String, __Value>) -> __Value {
    __Value::Object(obj.into_iter().map(|(k, v)| (k, normalize(v))).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_literal_like() {
        // Boolean literals
        assert!(is_literal_like("true"));
        assert!(is_literal_like("false"));
        assert!(is_literal_like("null"));

        // Numeric literals
        assert!(is_literal_like("0"));
        assert!(is_literal_like("42"));
        assert!(is_literal_like("-5"));
        assert!(is_literal_like("3.14"));
        assert!(is_literal_like("-3.14"));
        assert!(is_literal_like("1e6"));
        assert!(is_literal_like("1.5e-10"));

        // Not literals
        assert!(!is_literal_like("hello"));
        assert!(!is_literal_like("True")); // Wrong case
        assert!(!is_literal_like("null ")); // Trailing space
        assert!(!is_literal_like("05")); // Leading zero (not valid number)
        assert!(!is_literal_like("007")); // Leading zeros
    }

    #[test]
    fn test_is_numeric_like() {
        // Valid numbers
        assert!(is_numeric_like("0"));
        assert!(is_numeric_like("42"));
        assert!(is_numeric_like("-5"));
        assert!(is_numeric_like("3.14"));
        assert!(is_numeric_like("-3.14"));
        assert!(is_numeric_like("0.5"));

        // Invalid - leading zeros
        assert!(!is_numeric_like("05"));
        assert!(!is_numeric_like("007"));
        assert!(!is_numeric_like("0123"));

        // Not numbers
        assert!(!is_numeric_like("hello"));
        assert!(!is_numeric_like(""));
        assert!(!is_numeric_like("-"));
        assert!(!is_numeric_like("-."));
    }

    #[test]
    fn test_needs_quoting_empty() {
        let comma = Delimiter::Comma;
        assert!(needs_quoting("", QuotingContext::Value, comma));
        assert!(needs_quoting("", QuotingContext::Key, comma));
    }

    #[test]
    fn test_needs_quoting_whitespace() {
        let comma = Delimiter::Comma;
        assert!(needs_quoting(" hello", QuotingContext::Value, comma));
        assert!(needs_quoting("hello ", QuotingContext::Value, comma));
        assert!(needs_quoting(" hello ", QuotingContext::Value, comma));
        assert!(needs_quoting("  ", QuotingContext::Value, comma));
    }

    #[test]
    fn test_needs_quoting_literals() {
        let comma = Delimiter::Comma;
        assert!(needs_quoting("true", QuotingContext::Value, comma));
        assert!(needs_quoting("false", QuotingContext::Value, comma));
        assert!(needs_quoting("null", QuotingContext::Value, comma));
        assert!(needs_quoting("42", QuotingContext::Value, comma));
        assert!(needs_quoting("-3.14", QuotingContext::Value, comma));
    }

    #[test]
    fn test_needs_quoting_structural() {
        let comma = Delimiter::Comma;
        assert!(needs_quoting("[", QuotingContext::Value, comma));
        assert!(needs_quoting("]", QuotingContext::Value, comma));
        assert!(needs_quoting("{", QuotingContext::Value, comma));
        assert!(needs_quoting("}", QuotingContext::Value, comma));
        assert!(needs_quoting("[5]", QuotingContext::Value, comma));
        assert!(needs_quoting("{key}", QuotingContext::Value, comma));
        assert!(needs_quoting("a[0]", QuotingContext::Value, comma));
    }

    #[test]
    fn test_needs_quoting_delimiters() {
        assert!(needs_quoting("a,b", QuotingContext::Value, Delimiter::Comma));
        assert!(needs_quoting("a|b", QuotingContext::Value, Delimiter::Pipe));
        assert!(needs_quoting("a\tb", QuotingContext::Value, Delimiter::Tab));

        // Non-active delimiters don't require quoting
        assert!(!needs_quoting("a,b", QuotingContext::Value, Delimiter::Pipe));
        assert!(!needs_quoting("a|b", QuotingContext::Value, Delimiter::Comma));
    }

    #[test]
    fn test_needs_quoting_colon() {
        let comma = Delimiter::Comma;
        assert!(needs_quoting("a:b", QuotingContext::Value, comma));
        assert!(needs_quoting("key:", QuotingContext::Value, comma));
    }

    #[test]
    fn test_needs_quoting_hyphen() {
        let comma = Delimiter::Comma;
        
        // Value context: hyphen requires quoting
        assert!(needs_quoting("-", QuotingContext::Value, comma));
        assert!(needs_quoting("- item", QuotingContext::Value, comma));
        
        // Key context: hyphen doesn't require quoting
        assert!(!needs_quoting("-lead", QuotingContext::Key, comma));
        
        // Safe hyphens
        assert!(!needs_quoting("hello-world", QuotingContext::Value, comma));
        assert!(!needs_quoting("x-axis", QuotingContext::Value, comma));
    }

    #[test]
    fn test_needs_quoting_control_chars() {
        let comma = Delimiter::Comma;
        assert!(needs_quoting("hello\nworld", QuotingContext::Value, comma));
        assert!(needs_quoting("tab\there", QuotingContext::Value, comma));
        assert!(needs_quoting("line\\break", QuotingContext::Value, comma));
        assert!(needs_quoting("say \"hi\"", QuotingContext::Value, comma));
    }

    #[test]
    fn test_needs_quoting_safe_strings() {
        let comma = Delimiter::Comma;
        assert!(!needs_quoting("hello", QuotingContext::Value, comma));
        assert!(!needs_quoting("hello_world", QuotingContext::Value, comma));
        assert!(!needs_quoting("HelloWorld", QuotingContext::Value, comma));
        assert!(!needs_quoting("hello123", QuotingContext::Value, comma));
        assert!(!needs_quoting("user.name", QuotingContext::Key, comma));
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_string("tab\there"), "tab\\there");
        assert_eq!(escape_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_string("C:\\path"), "C:\\\\path");
        assert_eq!(escape_string("line1\r\nline2"), "line1\\r\\nline2");
    }

    #[test]
    fn test_unescape_string() {
        assert_eq!(unescape_string("hello").unwrap(), "hello");
        assert_eq!(unescape_string("hello\\nworld").unwrap(), "hello\nworld");
        assert_eq!(unescape_string("tab\\there").unwrap(), "tab\there");
        assert_eq!(unescape_string("say \\\"hi\\\"").unwrap(), "say \"hi\"");
        assert_eq!(unescape_string("C:\\\\path").unwrap(), "C:\\path");
        assert_eq!(
            unescape_string("line1\\r\\nline2").unwrap(),
            "line1\r\nline2"
        );
    }

    #[test]
    fn test_unescape_invalid() {
        assert!(unescape_string("\\x").is_err());
        assert!(unescape_string("\\").is_err());
        assert!(unescape_string("hello\\").is_err());
    }

    #[test]
    fn test_format_quoted_string() {
        assert_eq!(format_quoted_string("hello"), r#""hello""#);
        assert_eq!(format_quoted_string("hello\nworld"), r#""hello\nworld""#);
        assert_eq!(format_quoted_string(""), r#""""#);
    }

    #[test]
    fn test_round_trip_escape_unescape() {
        let test_strings = vec![
            "hello",
            "hello\nworld",
            "tab\there",
            "say \"hi\"",
            "C:\\path",
            "line1\r\nline2",
            "",
            "123",
        ];

        for s in test_strings {
            let escaped = escape_string(s);
            let unescaped = unescape_string(&escaped).unwrap();
            assert_eq!(s, unescaped, "Round trip failed for: {}", s);
        }
    }
}