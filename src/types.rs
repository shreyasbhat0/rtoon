use std::fmt;

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Delimiter {
    Comma,
    Tab,
    Pipe,
}

impl Delimiter {
    pub fn as_char(&self) -> char {
        match self {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Pipe => '|',
        }
    }

    pub fn as_metadata_str(&self) -> &'static str {
        match self {
            Delimiter::Comma => "",
            Delimiter::Tab => "\t",
            Delimiter::Pipe => "|",
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            ',' => Some(Delimiter::Comma),
            '\t' => Some(Delimiter::Tab),
            '|' => Some(Delimiter::Pipe),
            _ => None,
        }
    }

    pub fn contains_in(&self, s: &str) -> bool {
        s.contains(self.as_char())
    }
}

impl Default for Delimiter {
    fn default() -> Self {
        Delimiter::Comma
    }
}

impl fmt::Display for Delimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeOptions {
    pub delimiter: Delimiter,
    pub length_marker: Option<char>,
    pub indent: String,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            delimiter: Delimiter::Comma,
            length_marker: None,
            indent: "  ".to_string(),
        }
    }
}

impl EncodeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_delimiter(mut self, delimiter: Delimiter) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_length_marker(mut self, marker: char) -> Self {
        self.length_marker = Some(marker);
        self
    }

    pub fn with_indent(mut self, indent: impl Into<String>) -> Self {
        self.indent = indent.into();
        self
    }

    pub fn format_length(&self, length: usize) -> String {
        if let Some(marker) = self.length_marker {
            format!("{}{}", marker, length)
        } else {
            length.to_string()
        }
    }

    pub fn with_spaces(mut self, count: usize) -> Self {
        self.indent = " ".repeat(count);
        self
    }

    pub fn with_tabs(mut self) -> Self {
        self.indent = "\t".to_string();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeOptions {
    pub delimiter: Option<Delimiter>,
    pub strict: bool,
    pub coerce_types: bool,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            delimiter: None,
            strict: true,
            coerce_types: true,
        }
    }
}

impl DecodeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn with_delimiter(mut self, delimiter: Delimiter) -> Self {
        self.delimiter = Some(delimiter);
        self
    }
    pub fn with_coerce_types(mut self, coerce: bool) -> Self {
        self.coerce_types = coerce;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delimiter_conversion() {
        assert_eq!(Delimiter::Comma.as_char(), ',');
        assert_eq!(Delimiter::Tab.as_char(), '\t');
        assert_eq!(Delimiter::Pipe.as_char(), '|');
    }

    #[test]
    fn test_delimiter_from_char() {
        assert_eq!(Delimiter::from_char(','), Some(Delimiter::Comma));
        assert_eq!(Delimiter::from_char('\t'), Some(Delimiter::Tab));
        assert_eq!(Delimiter::from_char('|'), Some(Delimiter::Pipe));
        assert_eq!(Delimiter::from_char('x'), None);
    }

    #[test]
    fn test_delimiter_contains() {
        assert!(Delimiter::Comma.contains_in("a,b,c"));
        assert!(Delimiter::Tab.contains_in("a\tb\tc"));
        assert!(Delimiter::Pipe.contains_in("a|b|c"));
        assert!(!Delimiter::Comma.contains_in("abc"));
    }

    #[test]
    fn test_encode_options_length_marker() {
        let opts = EncodeOptions::new().with_length_marker('#');
        assert_eq!(opts.format_length(5), "#5");

        let opts = EncodeOptions::new();
        assert_eq!(opts.format_length(5), "5");
    }

    #[test]
    fn test_encode_options_indent() {
        let opts = EncodeOptions::new().with_spaces(4);
        assert_eq!(opts.indent, "    ");

        let opts = EncodeOptions::new().with_tabs();
        assert_eq!(opts.indent, "\t");

        let opts = EncodeOptions::new().with_indent("   ");
        assert_eq!(opts.indent, "   ");
    }

    #[test]
    fn test_decode_options_coerce_types() {
        let opts = DecodeOptions::new();
        assert!(opts.coerce_types);

        let opts = DecodeOptions::new().with_coerce_types(false);
        assert!(!opts.coerce_types);

        let opts = DecodeOptions::new().with_coerce_types(true);
        assert!(opts.coerce_types);
    }
}
