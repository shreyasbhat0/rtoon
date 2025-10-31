use crate::types::Delimiter;

pub const STRUCTURAL_CHARS: &[char] = &['[', ']', '{', '}', ':', '-'];

pub const KEYWORDS: &[&str] = &["null", "true", "false"];

pub const DEFAULT_INDENT: usize = 2;

pub const DEFAULT_DELIMITER: Delimiter = Delimiter::Comma;

pub const MAX_DEPTH: usize = 256;

#[inline]
pub fn is_structural_char(ch: char) -> bool {
    STRUCTURAL_CHARS.contains(&ch)
}

#[inline]
pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_structural_char() {
        assert!(is_structural_char('['));
        assert!(is_structural_char(']'));
        assert!(is_structural_char('{'));
        assert!(is_structural_char('}'));
        assert!(is_structural_char(':'));
        assert!(is_structural_char('-'));
        assert!(!is_structural_char('a'));
        assert!(!is_structural_char(','));
    }

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("null"));
        assert!(is_keyword("true"));
        assert!(is_keyword("false"));
        assert!(!is_keyword("hello"));
        assert!(!is_keyword("TRUE"));
    }
}
