use crate::{
    types::Delimiter,
    utils::literal,
};

pub fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for ch in s.chars() {
        match ch {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            _ => result.push(ch),
        }
    }

    result
}

pub fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn needs_quoting(s: &str, delimiter: Delimiter) -> bool {
    if s.is_empty() {
        return true;
    }

    if literal::is_literal_like(s) {
        return true;
    }

    if s.chars().any(|ch| literal::is_structural_char(ch)) {
        return true;
    }

    if s.contains(delimiter.as_char()) {
        return true;
    }

    if s.contains('\n') || s.contains('\r') {
        return true;
    }

    if s.contains(' ') || s.contains('\t') {
        return true;
    }

    if s.starts_with("- ") {
        return true;
    }

    false
}

pub fn quote_string(s: &str) -> String {
    format!("\"{}\"", escape_string(s))
}

pub fn split_by_delimiter(s: &str, delimiter: Delimiter) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = s.chars().peekable();
    let delim_char = delimiter.as_char();

    while let Some(ch) = chars.next() {
        if ch == '"' && (current.is_empty() || !current.ends_with('\\')) {
            in_quotes = !in_quotes;
            current.push(ch);
        } else if ch == delim_char && !in_quotes {
            result.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_unescape_string() {
        assert_eq!(unescape_string("hello"), "hello");
        assert_eq!(unescape_string("hello\\nworld"), "hello\nworld");
        assert_eq!(unescape_string("say \\\"hi\\\""), "say \"hi\"");
        assert_eq!(unescape_string("back\\\\slash"), "back\\slash");
    }

    #[test]
    fn test_needs_quoting() {
        let comma = Delimiter::Comma;

        assert!(needs_quoting("", comma));

        assert!(needs_quoting("true", comma));
        assert!(needs_quoting("false", comma));
        assert!(needs_quoting("null", comma));
        assert!(needs_quoting("123", comma));

        assert!(needs_quoting("hello[world]", comma));
        assert!(needs_quoting("key:value", comma));

        assert!(needs_quoting("a,b", comma));
        assert!(!needs_quoting("a,b", Delimiter::Pipe));

        assert!(needs_quoting("hello world", comma));
        assert!(needs_quoting(" hello", comma));
        assert!(needs_quoting("hello ", comma));

        assert!(!needs_quoting("hello", comma));
        assert!(!needs_quoting("world", comma));
        assert!(!needs_quoting("helloworld", comma));
    }

    #[test]
    fn test_quote_string() {
        assert_eq!(quote_string("hello"), "\"hello\"");
        assert_eq!(quote_string("hello\nworld"), "\"hello\\nworld\"");
    }

    #[test]
    fn test_split_by_delimiter() {
        let comma = Delimiter::Comma;

        assert_eq!(split_by_delimiter("a,b,c", comma), vec!["a", "b", "c"]);

        assert_eq!(split_by_delimiter("a, b, c", comma), vec!["a", "b", "c"]);

        assert_eq!(split_by_delimiter("\"a,b\",c", comma), vec!["\"a,b\"", "c"]);
    }
}
