### TOON (Token-Oriented Object Notation) — Rust

[![Crates.io](https://img.shields.io/crates/v/rtoon)](https://crates.io/crates/rtoon)

TOON is a compact, human-readable format designed to pass structured data to Large Language Models with fewer tokens than JSON. This is a Rust implementation.

### Why TOON?

LLM tokens cost money, and JSON is verbose. TOON conveys the same information with 30–60% fewer tokens than JSON.

JSON (example):

```json
{
  "users": [
    { "id": 1, "name": "Alice", "role": "admin" },
    { "id": 2, "name": "Bob", "role": "user" }
  ]
}
```

TOON:

```
users[2]{id,name,role}:
  1,Alice,admin
  2,Bob,user
```

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rtoon = "0.1.0"
serde_json = "1.0"
```

### Quick Start

```rust
use rtoon::{encode_default};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = json!({
        "user": {
            "id": 123,
            "name": "Ada",
            "tags": ["reading", "gaming"],
            "active": true
        }
    });

    let toon = encode_default(&data)?;
    println!("{}", toon);
    Ok(())
}
```

Output:

```
user:
  active: true
  id: 123
  name: Ada
  tags[2]: reading,gaming
```

### Features

- Token-efficient: 30–60% fewer tokens than JSON
- Human-readable: indentation-based structure (YAML-like)
- Three array formats: inline (primitive), tabular (uniform objects), list (mixed/nested)
- Type-safe: expressive options and enums; integrates with `serde_json`
- Customizable: delimiter (comma/tab/pipe), length markers, and indentation string
- Round-trip: encoder + decoder with property-style tests
- Production-focused: clear error types via `ToonError`

### Examples

#### Objects

```rust
use rtoon::encode_default;
use serde_json::json;

let data = json!({
    "id": 123,
    "name": "Ada",
    "active": true
});
println!("{}", encode_default(&data).unwrap());
```

Output:

```
active: true
id: 123
name: Ada
```

#### Arrays of Objects (Tabular)

```rust
use rtoon::encode_default;
use serde_json::json;

let data = json!({
    "items": [
        { "sku": "A1", "qty": 2, "price": 9.99 },
        { "sku": "B2", "qty": 1, "price": 14.5 }
    ]
});
println!("{}", encode_default(&data).unwrap());
```

Output:

```
items[2]{price,qty,sku}:
  9.99,2,A1
  14.5,1,B2
```

#### Custom Delimiters

Use tab or pipe delimiters to avoid quoting and save more tokens.

```rust
use rtoon::{encode, types::{EncodeOptions, Delimiter}};
use serde_json::json;

let data = json!({ "tags": ["reading", "gaming", "coding"] });
let opts = EncodeOptions::new().with_delimiter(Delimiter::Tab);
println!("{}", encode(&data, &opts).unwrap());
```

Possible output with tab delimiter (visualized as ␉):

```
tags[3\t]: reading\tgaming\tcoding
```

#### Length Markers

Prefix array lengths for clarity:

```rust
use rtoon::{encode, types::EncodeOptions};
use serde_json::json;

let data = json!({"tags": ["a", "b", "c"]});
let opts = EncodeOptions::new().with_length_marker('#');
println!("{}", encode(&data, &opts).unwrap());
```

Output:

```
tags[#3]: a,b,c
```

#### Decoding

```rust
use rtoon::decode_default;
use serde_json::json;

let input = "users[2]{id,name,role}:\n  1,Alice,admin\n  2,Bob,user";
let value = decode_default(input).unwrap();
assert_eq!(value, json!({
    "users": [
        {"id": 1, "name": "Alice", "role": "admin"},
        {"id": 2, "name": "Bob", "role": "user"}
    ]
}));
```

### API

#### Encoding

```rust
pub fn encode(value: &serde_json::Value, options: &EncodeOptions) -> ToonResult<String>
pub fn encode_default(value: &serde_json::Value) -> ToonResult<String>
```

#### Decoding

```rust
pub fn decode(input: &str, options: &DecodeOptions) -> ToonResult<serde_json::Value>
pub fn decode_default(input: &str) -> ToonResult<serde_json::Value>
```

#### EncodeOptions

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeOptions {
    pub delimiter: Delimiter,         // default: Delimiter::Comma
    pub length_marker: Option<char>,  // default: None
    pub indent: String,               // default: "  " (2 spaces)
}

impl EncodeOptions {
    pub fn new() -> Self
    pub fn with_delimiter(self, delimiter: Delimiter) -> Self
    pub fn with_length_marker(self, marker: char) -> Self
    pub fn with_indent(self, indent: impl Into<String>) -> Self
}
```

#### DecodeOptions

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeOptions {
    pub delimiter: Option<Delimiter>, // auto-detect if None
    pub strict: bool,                 // default: false
}

impl DecodeOptions {
    pub fn new() -> Self
    pub fn with_strict(self, strict: bool) -> Self
    pub fn with_delimiter(self, delimiter: Delimiter) -> Self
}
```

#### Delimiter

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter { Comma, Tab, Pipe }
```

### Format Overview

- Objects: `key: value` with 2-space indentation for nesting
- Primitive arrays: inline with count, e.g., `tags[3]: a,b,c`
- Arrays of objects: tabular header, e.g., `items[2]{id,name}:\n  ...`
- Mixed arrays: list format with `- ` prefix
- Quoting: only when necessary (special chars, ambiguity, keywords like `true`, `null`)

### License

MIT © 2025

### See Also

Original JS/TS implementation: @byjohann/toon


