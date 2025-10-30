use crate::error::ToonResult;
use crate::utils::{format_quoted_string, needs_quoting, QuotingContext};
use crate::types::EncodeOptions;

pub struct Writer {
    output: String,
    options: EncodeOptions,
}

impl Writer {
    pub fn new(options: EncodeOptions) -> Self {
        Self {
            output: String::new(),
            options,
        }
    }

    pub fn write_str(&mut self, s: &str) -> ToonResult<()> {
        self.output.push_str(s);
        Ok(())
    }

    pub fn write_delimiter(&mut self) -> ToonResult<()> {
        self.output.push(self.options.delimiter.as_char());
        Ok(())
    }

    pub fn write_newline(&mut self) -> ToonResult<()> {
        self.output.push('\n');
        Ok(())
    }

    pub fn write_indent(&mut self, level: usize) -> ToonResult<()> {
        for _ in 0..level {
            self.output.push_str(&self.options.indent);
        }
        Ok(())
    }

    pub fn write_null(&mut self) -> ToonResult<()> {
        self.output.push_str("null");
        Ok(())
    }

    pub fn write_bool(&mut self, b: bool) -> ToonResult<()> {
        self.output.push_str(if b { "true" } else { "false" });
        Ok(())
    }

    pub fn write_number(&mut self, n: &serde_json::Number) -> ToonResult<()> {
        self.output.push_str(&n.to_string());
        Ok(())
    }

    pub fn write_string(&mut self, s: &str, _depth: usize) -> ToonResult<()> {
        if needs_quoting(s, QuotingContext::Value, self.options.delimiter) {
            self.write_quoted_string(s)
        } else {
            self.output.push_str(s);
            Ok(())
        }
    }

    pub fn write_quoted_string(&mut self, s: &str) -> ToonResult<()> {
        self.output.push_str(&format_quoted_string(s));
        Ok(())
    }

    pub fn needs_quoting(&self, s: &str) -> bool {
        needs_quoting(s, QuotingContext::Value, self.options.delimiter)
    }

    pub fn write_key(&mut self, key: &str) -> ToonResult<()> {
        if needs_quoting(key, QuotingContext::Key, self.options.delimiter) {
            self.write_quoted_string(key)
        } else {
            self.output.push_str(key);
            Ok(())
        }
    }

    pub fn write_array_header(
        &mut self,
        key: Option<&str>,
        length: usize,
        fields: Option<&[String]>,
        _depth: usize,
    ) -> ToonResult<()> {
        if let Some(k) = key {
            self.write_key(k)?;
        }

        self.output.push('[');
        self.output
            .push_str(&self.options.format_length(length));

        if self.options.delimiter != crate::types::Delimiter::Comma {
            self.output.push(self.options.delimiter.as_char());
        }

        self.output.push(']');

        if let Some(field_list) = fields {
            self.write_field_list(field_list)?;
        }

        self.output.push(':');
        Ok(())
    }

    pub fn write_empty_array_with_key(&mut self, key: Option<&str>) -> ToonResult<()> {
        self.write_array_header(key, 0, None, 0)?;
        Ok(())
    }

    pub fn write_field_list(&mut self, keys: &[String]) -> ToonResult<()> {
        self.output.push('{');
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                self.output.push(self.options.delimiter.as_char());
            }
            if needs_quoting(key, QuotingContext::Header, self.options.delimiter) {
                self.write_quoted_string(key)?;
            } else {
                self.output.push_str(key);
            }
        }
        self.output.push('}');
        Ok(())
    }

    pub fn finish(self) -> String {
        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Delimiter;

    #[test]
    fn test_writer_basic() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_str("hello").unwrap();
        assert_eq!(writer.finish(), "hello");
    }

    #[test]
    fn test_write_null() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_null().unwrap();
        assert_eq!(writer.finish(), "null");
    }

    #[test]
    fn test_write_bool() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_bool(true).unwrap();
        writer.write_str(",").unwrap();
        writer.write_bool(false).unwrap();
        assert_eq!(writer.finish(), "true,false");
    }

    #[test]
    fn test_write_number() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        let num = serde_json::Number::from(42);
        writer.write_number(&num).unwrap();
        assert_eq!(writer.finish(), "42");
    }

    #[test]
    fn test_write_string_no_quoting() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_string("hello", 0).unwrap();
        assert_eq!(writer.finish(), "hello");
    }

    #[test]
    fn test_write_string_with_quoting() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        // Leading space requires quoting
        writer.write_string(" hello", 0).unwrap();
        assert_eq!(writer.finish(), "\" hello\"");
    }

    #[test]
    fn test_write_string_internal_spaces_no_quoting() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        // Internal spaces don't require quoting
        writer.write_string("hello world", 0).unwrap();
        assert_eq!(writer.finish(), "hello world");
    }

    #[test]
    fn test_write_quoted_string() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_quoted_string("hello\nworld").unwrap();
        assert_eq!(writer.finish(), r#""hello\nworld""#);
    }

    #[test]
    fn test_write_delimiter() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_str("a").unwrap();
        writer.write_delimiter().unwrap();
        writer.write_str("b").unwrap();
        assert_eq!(writer.finish(), "a,b");
    }

    #[test]
    fn test_write_delimiter_pipe() {
        let opts = EncodeOptions::new().with_delimiter(Delimiter::Pipe);
        let mut writer = Writer::new(opts);

        writer.write_str("a").unwrap();
        writer.write_delimiter().unwrap();
        writer.write_str("b").unwrap();
        assert_eq!(writer.finish(), "a|b");
    }

    #[test]
    fn test_write_indent() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_indent(0).unwrap();
        writer.write_str("level0").unwrap();
        writer.write_newline().unwrap();
        writer.write_indent(1).unwrap();
        writer.write_str("level1").unwrap();
        assert_eq!(writer.finish(), "level0\n  level1");
    }

    #[test]
    fn test_write_array_header_simple() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer
            .write_array_header(Some("tags"), 3, None, 0)
            .unwrap();
        assert_eq!(writer.finish(), "tags[3]:");
    }

    #[test]
    fn test_write_array_header_with_fields() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        let fields = vec!["id".to_string(), "name".to_string()];
        writer
            .write_array_header(Some("users"), 2, Some(&fields), 0)
            .unwrap();
        assert_eq!(writer.finish(), "users[2]{id,name}:");
    }

    #[test]
    fn test_write_array_header_pipe_delimiter() {
        let opts = EncodeOptions::new().with_delimiter(Delimiter::Pipe);
        let mut writer = Writer::new(opts);

        writer
            .write_array_header(Some("tags"), 3, None, 0)
            .unwrap();
        assert_eq!(writer.finish(), "tags[3|]:");
    }

    #[test]
    fn test_write_empty_array() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_empty_array_with_key(Some("items")).unwrap();
        assert_eq!(writer.finish(), "items[0]:");
    }

    #[test]
    fn test_write_field_list() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        let fields = vec!["id".to_string(), "name".to_string()];
        writer.write_field_list(&fields).unwrap();
        assert_eq!(writer.finish(), "{id,name}");
    }

    #[test]
    fn test_write_field_list_with_pipe() {
        let opts = EncodeOptions::new().with_delimiter(Delimiter::Pipe);
        let mut writer = Writer::new(opts);

        let fields = vec!["id".to_string(), "name".to_string()];
        writer.write_field_list(&fields).unwrap();
        assert_eq!(writer.finish(), "{id|name}");
    }

    #[test]
    fn test_write_key_with_special_chars() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_key("order:id").unwrap();
        assert_eq!(writer.finish(), r#""order:id""#);
    }

    #[test]
    fn test_write_key_safe() {
        let opts = EncodeOptions::default();
        let mut writer = Writer::new(opts);

        writer.write_key("name").unwrap();
        assert_eq!(writer.finish(), "name");
    }

    #[test]
    fn test_needs_quoting_helper() {
        let opts = EncodeOptions::default();
        let writer = Writer::new(opts);

        assert!(!writer.needs_quoting("hello"));
        assert!(writer.needs_quoting("hello,world"));
        assert!(writer.needs_quoting("true"));
        assert!(writer.needs_quoting(""));
    }
}