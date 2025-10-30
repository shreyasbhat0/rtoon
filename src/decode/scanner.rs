use crate::error::{ToonResult, ToonError};
use crate::types::Delimiter;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Colon,
    Dash,
    Newline,
    String(String),
    Number(f64),
    Integer(i64),
    Bool(bool),
    Null,
    Delimiter(Delimiter),
    Eof,
}

pub struct Scanner {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    active_delimiter: Option<Delimiter>,
    last_line_indent: usize,
}

impl Scanner {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            active_delimiter: None,
            last_line_indent: 0,
        }
    }

    pub fn set_active_delimiter(&mut self, delimiter: Option<Delimiter>) {
        self.active_delimiter = delimiter;
    }

    pub fn current_position(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    pub fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    pub fn count_leading_spaces(&self) -> usize {
        let mut idx = self.position;
        let mut count = 0;
        while let Some(&ch) = self.input.get(idx) {
            if ch == ' ' {
                count += 1;
                idx += 1;
            } else {
                break;
            }
        }
        count
    }

    /// If the current character is a newline, count spaces immediately after it (without advancing)
    pub fn count_spaces_after_newline(&self) -> usize {
        let mut idx = self.position;
        if self.input.get(idx) != Some(&'\n') {
            return 0;
        }
        idx += 1;
        let mut count = 0;
        while let Some(&ch) = self.input.get(idx) {
            if ch == ' ' {
                count += 1;
                idx += 1;
            } else {
                break;
            }
        }
        count
    }

    pub fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    pub fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.input.get(self.position) {
            self.position += 1;
            if *ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(*ch)
        } else {
            None
        }
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn scan_token(&mut self) -> ToonResult<Token> {
        if self.column == 1 {
            let mut count = 0;
            let mut idx = self.position;
            while let Some(&ch) = self.input.get(idx) {
                if ch == ' ' {
                    count += 1;
                    idx += 1;
                } else {
                    break;
                }
            }
            self.last_line_indent = count;
        }

        self.skip_whitespace();

        match self.peek() {
            None => Ok(Token::Eof),
            Some('\n') => {
                self.advance();
                Ok(Token::Newline)
            }
            Some('[') => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            Some(']') => {
                self.advance();
                Ok(Token::RightBracket)
            }
            Some('{') => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RightBrace)
            }
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some('-') => {
                self.advance();
                if let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        let num_str = self.scan_number_string(true)?;
                        return self.parse_number(&num_str);
                    }
                }
                Ok(Token::Dash)
            }
            Some(',') => {
                if matches!(self.active_delimiter, Some(Delimiter::Comma)) {
                    self.advance();
                    Ok(Token::Delimiter(Delimiter::Comma))
                } else {
                    self.scan_unquoted_string()
                }
            }
            Some('|') => {
                if matches!(self.active_delimiter, Some(Delimiter::Pipe)) {
                    self.advance();
                    Ok(Token::Delimiter(Delimiter::Pipe))
                } else {
                    self.scan_unquoted_string()
                }
            }
            Some('\t') => {
                if matches!(self.active_delimiter, Some(Delimiter::Tab)) {
                    self.advance();
                    Ok(Token::Delimiter(Delimiter::Tab))
                } else {
                    self.scan_unquoted_string()
                }
            }
            Some('"') => self.scan_quoted_string(),
            Some(ch) if ch.is_ascii_digit() => {
                let num_str = self.scan_number_string(false)?;
                self.parse_number(&num_str)
            }
            Some(_) => self.scan_unquoted_string(),
        }
    }

    fn scan_quoted_string(&mut self) -> ToonResult<Token> {
        self.advance();

        let mut value = String::new();
        let mut escaped = false;

        while let Some(ch) = self.advance() {
            if escaped {
                match ch {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    _ => {
                        value.push('\\');
                        value.push(ch);
                    }
                }
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                return Ok(Token::String(value));
            } else {
                value.push(ch);
            }
        }

        Err(ToonError::UnexpectedEof)
    }

    fn scan_unquoted_string(&mut self) -> ToonResult<Token> {
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch == '\n'
                || ch == ' '
                || ch == ':'
                || ch == '['
                || ch == ']'
                || ch == '{'
                || ch == '}'
            {
                break;
            }

            if let Some(active) = self.active_delimiter {
                if (active == Delimiter::Comma && ch == ',')
                    || (active == Delimiter::Pipe && ch == '|')
                    || (active == Delimiter::Tab && ch == '\t')
                {
                    break;
                }
            }
            value.push(ch);
            self.advance();
        }

        let value = if value.len() == 1 && (value == "," || value == "|" || value == "\t") {
            value
        } else {
            value.trim_end().to_string()
        };

        match value.as_str() {
            "null" => Ok(Token::Null),
            "true" => Ok(Token::Bool(true)),
            "false" => Ok(Token::Bool(false)),
            _ => Ok(Token::String(value)),
        }
    }

    pub fn get_last_line_indent(&self) -> usize {
        self.last_line_indent
    }

    fn scan_number_string(&mut self, negative: bool) -> ToonResult<String> {
        let mut num_str = if negative {
            String::from("-")
        } else {
            String::new()
        };

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-'
            {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(num_str)
    }

    fn parse_number(&self, s: &str) -> ToonResult<Token> {
        if s.contains('.') || s.contains('e') || s.contains('E') {
            if let Ok(f) = s.parse::<f64>() {
                Ok(Token::Number(f))
            } else {
                Ok(Token::String(s.to_string()))
            }
        } else if let Ok(i) = s.parse::<i64>() {
            Ok(Token::Integer(i))
        } else {
            Ok(Token::String(s.to_string()))
        }
    }

    pub fn detect_delimiter(&mut self) -> Option<Delimiter> {
        let saved_pos = self.position;

        while let Some(ch) = self.peek() {
            match ch {
                ',' => {
                    self.position = saved_pos;
                    return Some(Delimiter::Comma);
                }
                '|' => {
                    self.position = saved_pos;
                    return Some(Delimiter::Pipe);
                }
                '\t' => {
                    self.position = saved_pos;
                    return Some(Delimiter::Tab);
                }
                '\n' | ':' | '[' | ']' | '{' | '}' => break,
                _ => {
                    self.advance();
                }
            }
        }

        self.position = saved_pos;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_structural_tokens() {
        let mut scanner = Scanner::new("[]{}:-");
        assert_eq!(scanner.scan_token().unwrap(), Token::LeftBracket);
        assert_eq!(scanner.scan_token().unwrap(), Token::RightBracket);
        assert_eq!(scanner.scan_token().unwrap(), Token::LeftBrace);
        assert_eq!(scanner.scan_token().unwrap(), Token::RightBrace);
        assert_eq!(scanner.scan_token().unwrap(), Token::Colon);
        assert_eq!(scanner.scan_token().unwrap(), Token::Dash);
    }

    #[test]
    fn test_scan_numbers() {
        let mut scanner = Scanner::new("42 3.14 -5");
        assert_eq!(scanner.scan_token().unwrap(), Token::Integer(42));
        assert_eq!(scanner.scan_token().unwrap(), Token::Number(3.14));
        assert_eq!(scanner.scan_token().unwrap(), Token::Integer(-5));
    }

    #[test]
    fn test_scan_booleans() {
        let mut scanner = Scanner::new("true false");
        assert_eq!(scanner.scan_token().unwrap(), Token::Bool(true));
        assert_eq!(scanner.scan_token().unwrap(), Token::Bool(false));
    }

    #[test]
    fn test_scan_null() {
        let mut scanner = Scanner::new("null");
        assert_eq!(scanner.scan_token().unwrap(), Token::Null);
    }

    #[test]
    fn test_scan_quoted_string() {
        let mut scanner = Scanner::new(r#""hello world""#);
        assert_eq!(
            scanner.scan_token().unwrap(),
            Token::String("hello world".to_string())
        );
    }

    #[test]
    fn test_scan_escaped_string() {
        let mut scanner = Scanner::new(r#""hello\nworld""#);
        assert_eq!(
            scanner.scan_token().unwrap(),
            Token::String("hello\nworld".to_string())
        );
    }

    #[test]
    fn test_scan_unquoted_string() {
        let mut scanner = Scanner::new("hello");
        assert_eq!(
            scanner.scan_token().unwrap(),
            Token::String("hello".to_string())
        );
    }

    #[test]
    fn test_detect_delimiter() {
        let mut scanner = Scanner::new("a,b,c");
        assert_eq!(scanner.detect_delimiter(), Some(Delimiter::Comma));

        let mut scanner = Scanner::new("a|b|c");
        assert_eq!(scanner.detect_delimiter(), Some(Delimiter::Pipe));

        let mut scanner = Scanner::new("a\tb\tc");
        assert_eq!(scanner.detect_delimiter(), Some(Delimiter::Tab));
    }
}