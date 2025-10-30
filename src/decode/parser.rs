use crate::decode::scanner::{Scanner, Token};
use crate::error::{ToonResult, ToonError};
use crate::types::{DecodeOptions, Delimiter};
use serde_json::{Map, Value};

pub struct Parser {
    scanner: Scanner,
    current_token: Token,
    _options: DecodeOptions,
    delimiter: Option<Delimiter>,
}

impl Parser {
    pub fn new(input: &str, options: DecodeOptions) -> Self {
        let mut scanner = Scanner::new(input);
        let chosen_delim = options.delimiter;
        scanner.set_active_delimiter(chosen_delim);
        let current_token = scanner.scan_token().unwrap_or(Token::Eof);

        Self { scanner, current_token, delimiter: chosen_delim, _options: options }
    }

    pub fn parse(&mut self) -> ToonResult<Value> {
        self.parse_value()
    }

    fn advance(&mut self) -> ToonResult<()> {
        self.current_token = self.scanner.scan_token()?;
        Ok(())
    }

    fn skip_newlines(&mut self) -> ToonResult<()> {
        while matches!(self.current_token, Token::Newline) {
            self.advance()?;
        }
        Ok(())
    }

    fn parse_value(&mut self) -> ToonResult<Value> {
        self.skip_newlines()?;

        match &self.current_token {
            Token::Null => {
                self.advance()?;
                Ok(Value::Null)
            }
            Token::Bool(b) => {
                let val = *b;
                self.advance()?;
                Ok(Value::Bool(val))
            }
            Token::Integer(i) => {
                let val = *i;
                self.advance()?;
                Ok(serde_json::Number::from(val).into())
            }
            Token::Number(n) => {
                let val = *n;
                self.advance()?;
                Ok(serde_json::Number::from_f64(val)
                    .ok_or_else(|| {
                        ToonError::InvalidInput(format!("Invalid number: {}", val))
                    })?
                    .into())
            }
            Token::String(s) => {
                let first = s.clone();
                self.advance()?;

                match &self.current_token {
                    Token::Colon | Token::LeftBracket => self.parse_object_with_initial_key(first),
                    _ => {
                        let mut accumulated = first;
                        loop {
                            match &self.current_token {
                                Token::String(next) => {
                                    if !accumulated.is_empty() { accumulated.push(' '); }
                                    accumulated.push_str(next);
                                    self.advance()?;
                                }
                                _ => break,
                            }
                        }
                        Ok(Value::String(accumulated))
                    }
                }
            }
            Token::LeftBracket => self.parse_root_array(),
            Token::Eof => Ok(Value::Null),
            _ => self.parse_object(),
        }
    }

    fn parse_object(&mut self) -> ToonResult<Value> {
        let mut obj = Map::new();
        let mut base_indent: Option<usize> = None;

        loop {
            while matches!(self.current_token, Token::Newline) {
                self.advance()?;
            }

            if matches!(self.current_token, Token::Eof) {
                break;
            }

            let current_indent = self.scanner.get_last_line_indent();
            if let Some(expected) = base_indent {
                if current_indent != expected {
                    break;
                }
            } else {
                base_indent = Some(current_indent);
            }

            let key = match &self.current_token {
                Token::String(s) => s.clone(),
                _ => {
                    return Err(ToonError::InvalidInput(format!(
                        "Expected key, found {:?}",
                        self.current_token
                    )))
                }
            };
            self.advance()?;

            let value = if matches!(self.current_token, Token::LeftBracket) {
                self.parse_array()?
            } else {
                if !matches!(self.current_token, Token::Colon) {
                    return Err(ToonError::InvalidInput(format!(
                        "Expected ':' or '[', found {:?}",
                        self.current_token
                    )));
                }
                self.advance()?;
                self.parse_field_value()?
            };

            obj.insert(key, value);
        }

        Ok(Value::Object(obj))
    }

    fn parse_object_with_initial_key(&mut self, key: String) -> ToonResult<Value> {
        let mut obj = Map::new();

        let value = if matches!(self.current_token, Token::LeftBracket) {
            self.parse_array()?
        } else {
            if !matches!(self.current_token, Token::Colon) {
                return Err(ToonError::InvalidInput(format!(
                    "Expected ':' or '[', found {:?}",
                    self.current_token
                )));
            }
            self.advance()?;
            self.parse_field_value()?
        };

        obj.insert(key, value);

        self.skip_newlines()?;

        loop {
            if matches!(self.current_token, Token::Eof) {
                break;
            }

            let next_key = match &self.current_token {
                Token::String(s) => s.clone(),
                _ => break,
            };
            self.advance()?;

            let next_value = if matches!(self.current_token, Token::LeftBracket) {
                self.parse_array()?
            } else {
                if !matches!(self.current_token, Token::Colon) {
                    break;
                }
                self.advance()?;
                self.parse_field_value()?
            };

            obj.insert(next_key, next_value);
            self.skip_newlines()?;
        }

        Ok(Value::Object(obj))
    }

    fn parse_field_value(&mut self) -> ToonResult<Value> {
        match &self.current_token {
            Token::Newline => {
                self.parse_indented_object()
            }
            _ => self.parse_primitive(),
        }
    }

    fn parse_indented_object(&mut self) -> ToonResult<Value> {
        let mut obj = Map::new();

        loop {
            while matches!(self.current_token, Token::Newline) {
                self.advance()?;
            }

            if self.scanner.get_last_line_indent() == 0 || matches!(self.current_token, Token::Eof) {
                break;
            }

            let key = match &self.current_token {
                Token::String(s) => s.clone(),
                _ => {
                    return Err(ToonError::InvalidInput(format!(
                        "Expected key, found {:?}", self.current_token
                    )))
                }
            };

            self.advance()?;

            let value = if matches!(self.current_token, Token::LeftBracket) {
                self.parse_array()?
            } else {
                if !matches!(self.current_token, Token::Colon) {
                    return Err(ToonError::InvalidInput(format!(
                        "Expected ':' or '[', found {:?}", self.current_token
                    )));
                }
                self.advance()?;
                self.parse_field_value()?
            };

            obj.insert(key, value);
            while matches!(self.current_token, Token::Newline) {
                self.advance()?;
            }
        }

        Ok(Value::Object(obj))
    }

    fn parse_primitive(&mut self) -> ToonResult<Value> {
        match &self.current_token {
            Token::Null => {
                self.advance()?;
                Ok(Value::Null)
            }
            Token::Bool(b) => {
                let val = *b;
                self.advance()?;
                Ok(Value::Bool(val))
            }
            Token::Integer(i) => {
                let val = *i;
                self.advance()?;
                Ok(serde_json::Number::from(val).into())
            }
            Token::Number(n) => {
                let val = *n;
                self.advance()?;
                Ok(serde_json::Number::from_f64(val)
                    .ok_or_else(|| {
                        ToonError::InvalidInput(format!("Invalid number: {}", val))
                    })?
                    .into())
            }
            Token::String(s) => {
                let mut accumulated = s.clone();
                self.advance()?;

                loop {
                    match &self.current_token {
                        Token::String(next) => {
                            if !accumulated.is_empty() { accumulated.push(' '); }
                            accumulated.push_str(next);
                            self.advance()?;
                        }
                        _ => break,
                    }
                }

                Ok(Value::String(accumulated))
            }
            _ => Err(ToonError::InvalidInput(format!(
                "Expected primitive value, found {:?}",
                self.current_token
            ))),
        }
    }

    fn parse_array(&mut self) -> ToonResult<Value> {
        if !matches!(self.current_token, Token::LeftBracket) {
            return Err(ToonError::InvalidInput("Expected '['".to_string()));
        }
        self.advance()?;

        let length = self.parse_array_length()?;

        self.detect_or_consume_delimiter()?;

        if !matches!(self.current_token, Token::RightBracket) {
            return Err(ToonError::InvalidInput("Expected ']'".to_string()));
        }
        self.advance()?;

        if self.delimiter.is_none() {
            self.delimiter = Some(Delimiter::Comma);
        }
        self.scanner.set_active_delimiter(self.delimiter);

        let fields = if matches!(self.current_token, Token::LeftBrace) {
            Some(self.parse_field_list()?)
        } else {
            None
        };

        if !matches!(self.current_token, Token::Colon) {
            return Err(ToonError::InvalidInput("Expected ':'".to_string()));
        }
        self.advance()?;

        if length == 0 {
            return Ok(Value::Array(vec![]));
        }

        if let Some(fields) = fields {
            self.parse_tabular_array(length, fields)
        } else {
            self.parse_regular_array(length)
        }
    }

    fn parse_array_length(&mut self) -> ToonResult<usize> {
        if let Some(length_str) = match &self.current_token {
            Token::String(s) if s.starts_with('#') => Some(s[1..].to_string()),
            _ => None,
        } {
            self.advance()?;
            return length_str.parse::<usize>().map_err(|_| {
                ToonError::InvalidInput(format!("Invalid array length: {}", length_str))
            });
        }

        match &self.current_token {
            Token::Integer(i) => {
                let len = *i as usize;
                self.advance()?;
                Ok(len)
            }
            _ => Err(ToonError::InvalidInput(format!(
                "Expected array length, found {:?}",
                self.current_token
            ))),
        }
    }

    fn detect_or_consume_delimiter(&mut self) -> ToonResult<()> {
        match &self.current_token {
            Token::Delimiter(delim) => {
                if self.delimiter.is_none() {
                    self.delimiter = Some(*delim);
                }
                self.advance()?;
            }
            Token::String(s) if s == "," || s == "|" || s == "\t" => {
                let delim = if s == "," {
                    Delimiter::Comma
                } else if s == "|" {
                    Delimiter::Pipe
                } else {
                    Delimiter::Tab
                };
                if self.delimiter.is_none() {
                    self.delimiter = Some(delim);
                }
                self.advance()?;
            }
            _ => {}
        }
        self.scanner.set_active_delimiter(self.delimiter);
        Ok(())
    }

    fn parse_field_list(&mut self) -> ToonResult<Vec<String>> {
        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(ToonError::InvalidInput("Expected '{'".to_string()));
        }
        self.advance()?;

        let mut fields = Vec::new();

        loop {
            match &self.current_token {
                Token::String(s) => {
                    fields.push(s.clone());
                    self.advance()?;

                    if matches!(self.current_token, Token::Delimiter(_)) {
                        self.advance()?;
                    } else if matches!(self.current_token, Token::RightBrace) {
                        break;
                    }
                }
                Token::RightBrace => break,
                _ => {
                    return Err(ToonError::InvalidInput(format!(
                        "Expected field name, found {:?}",
                        self.current_token
                    )))
                }
            }
        }

        if !matches!(self.current_token, Token::RightBrace) {
            return Err(ToonError::InvalidInput("Expected '}'".to_string()));
        }
        self.advance()?;

        Ok(fields)
    }

    fn parse_tabular_array(&mut self, length: usize, fields: Vec<String>) -> ToonResult<Value> {
        let mut rows = Vec::new();

        self.skip_newlines()?;

        self.scanner.set_active_delimiter(self.delimiter);

        for _ in 0..length {
            let mut row = Map::new();

            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    match &self.current_token {
                        Token::Delimiter(_) => {
                            self.advance()?;
                        }
                        Token::String(s) if s == "," || s == "|" || s == "\t" => {
                            self.advance()?;
                        }
                        other => {
                            return Err(ToonError::InvalidInput(format!(
                                "Expected delimiter, found {:?}", other
                            )));
                        }
                    }
                }

                let value = self.parse_primitive()?;
                row.insert(field.clone(), value);
            }

            rows.push(Value::Object(row));
            self.skip_newlines()?;
        }

        Ok(Value::Array(rows))
    }

    fn parse_regular_array(&mut self, length: usize) -> ToonResult<Value> {
        self.skip_newlines()?;

        self.scanner.set_active_delimiter(self.delimiter);

        if matches!(self.current_token, Token::Dash) {
            self.parse_nested_array(length)
        } else {
            self.parse_primitive_array(length)
        }
    }

    fn parse_primitive_array(&mut self, length: usize) -> ToonResult<Value> {
        let mut values = Vec::new();

        for i in 0..length {
            if i > 0 {
                match &self.current_token {
                    Token::Delimiter(_) => {
                        self.advance()?;
                    }
                    Token::String(s) if s == "," || s == "|" || s == "\t" => {
                        self.advance()?;
                    }
                    other => {
                        return Err(ToonError::InvalidInput(format!(
                            "Expected delimiter, found {:?}", other
                        )));
                    }
                }
            }

            values.push(self.parse_primitive()?);
        }

        Ok(Value::Array(values))
    }

    fn parse_nested_array(&mut self, length: usize) -> ToonResult<Value> {
        let mut items = Vec::new();

        for _ in 0..length {
            if !matches!(self.current_token, Token::Dash) {
                return Err(ToonError::InvalidInput(format!(
                    "Expected '-', found {:?}",
                    self.current_token
                )));
            }
            self.advance()?;

            let value = self.parse_field_value()?;
            items.push(value);
            self.skip_newlines()?;
        }

        Ok(Value::Array(items))
    }

    fn parse_root_array(&mut self) -> ToonResult<Value> {
        self.parse_array()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(input: &str) -> ToonResult<Value> {
        let mut parser = Parser::new(input, DecodeOptions::default());
        parser.parse()
    }

    #[test]
    fn test_parse_primitives() {
        assert_eq!(parse("null").unwrap(), json!(null));
        assert_eq!(parse("true").unwrap(), json!(true));
        assert_eq!(parse("false").unwrap(), json!(false));
        assert_eq!(parse("42").unwrap(), json!(42));
        assert_eq!(parse("3.14").unwrap(), json!(3.14));
        assert_eq!(parse("hello").unwrap(), json!("hello"));
    }

    #[test]
    fn test_parse_object() {
        let result = parse("name: Alice\nage: 30").unwrap();
        assert_eq!(result["name"], json!("Alice"));
        assert_eq!(result["age"], json!(30));
    }

    #[test]
    fn test_parse_primitive_array() {
        let result = parse("tags[3]: a,b,c").unwrap();
        assert_eq!(result["tags"], json!(["a", "b", "c"]));
    }

    #[test]
    fn test_parse_empty_array() {
        let result = parse("items[0]:").unwrap();
        assert_eq!(result["items"], json!([]));
    }

    #[test]
    fn test_parse_tabular_array() {
        let result = parse("users[2]{id,name}:\n  1,Alice\n  2,Bob").unwrap();
        assert_eq!(
            result["users"],
            json!([
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ])
        );
    }
}