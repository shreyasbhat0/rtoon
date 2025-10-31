<div align="center">

# 🦀 RToon

**Rust implementation of TOON (Token-Oriented Object Notation)**

*A compact, token-efficient format for structured data in LLM applications*

<img src="https://github.com/alpkeskin/gotoon/raw/main/.github/og.png" alt="TOON - Token-Oriented Object Notation" width="600">

[![Crates.io](https://img.shields.io/crates/v/rtoon)](https://crates.io/crates/rtoon)
[![CI](https://github.com/shreyasbhat0/toon-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/shreyasbhat0/toon-rs/actions/workflows/ci.yml)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen)](https://github.com/shreyasbhat0/toon-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

</div>

---

**Token-Oriented Object Notation** is a compact, human-readable format designed for passing structured data to Large Language Models with significantly reduced token usage. This is a Rust implementation of the TOON specification.


> [!TIP]
> Think of TOON as a translation layer: use JSON programmatically, convert to TOON for LLM input.

## Table of Contents

- [Why TOON?](#why-toon)
- [Key Features](#key-features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Examples](#examples)
  - [Objects](#objects)
  - [Primitive Arrays](#primitive-arrays)
  - [Arrays of Objects (Tabular)](#arrays-of-objects-tabular)
  - [Arrays of Arrays](#arrays-of-arrays)
  - [Mixed Arrays](#mixed-arrays)
  - [Custom Delimiters](#custom-delimiters)
  - [Length Markers](#length-markers)
  - [Empty Containers & Root Forms](#empty-containers--root-forms)
  - [Round-Trip Encoding](#round-trip-encoding)
  - [Strict Mode Decoding](#strict-mode-decoding)
- [API Reference](#api-reference)
- [Format Overview](#format-overview)
- [Specification](#specification)
- [Running Examples](#running-examples)
- [Contributing](#contributing)
- [License](#license)
- [See Also](#see-also)

## Why TOON?

AI is becoming cheaper and more accessible, but larger context windows allow for larger data inputs as well. **LLM tokens still cost money** – and standard JSON is verbose and token-expensive.

### JSON vs TOON Comparison

<details>
<summary><strong>📊 Click to see the token efficiency comparison</strong></summary>

**JSON** (verbose, token-heavy):
```json
{
  "users": [
    { "id": 1, "name": "Alice", "role": "admin" },
    { "id": 2, "name": "Bob", "role": "user" }
  ]
}
```

**TOON** (compact, token-efficient):
```toon
users[2]{id,name,role}:
  1,Alice,admin
  2,Bob,user
```

TOON conveys the same information with **30–60% fewer tokens**! 🎉

</details>

## Key Features

- 💸 **Token-efficient:** typically 30–60% fewer tokens than JSON
- 🤿 **LLM-friendly guardrails:** explicit lengths and fields enable validation
- 🍱 **Minimal syntax:** removes redundant punctuation (braces, brackets, most quotes)
- 📐 **Indentation-based structure:** like YAML, uses whitespace instead of braces
- 🧺 **Tabular arrays:** declare keys once, stream data as rows
- 🔄 **Round-trip support:** encode and decode with full fidelity
- 🛡️ **Type-safe:** integrates seamlessly with `serde_json::Value`
- ⚙️ **Customizable:** delimiter (comma/tab/pipe), length markers, and indentation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rtoon = "0.1.0"
serde_json = "1.0"
```

## Quick Start

```rust
use rtoon::encode_default;
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

**Output:**

```toon
user:
  active: true
  id: 123
  name: Ada
  tags[2]: reading,gaming
```

---

## Examples

> **📝 Note:** All examples in this section are taken from the `examples/` directory. Run `cargo run --example examples` to see them in action.

### Objects

Simple objects encode as key-value pairs:

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

**Output:**

```toon
active: true
id: 123
name: Ada
```

Nested objects use indentation:

```rust
let nested = json!({
    "user": { "id": 123, "name": "Ada" }
});
println!("{}", encode_default(&nested).unwrap());
```

**Output:**

```toon
user:
  id: 123
  name: Ada
```

### Primitive Arrays

Primitive arrays are inline with count and delimiter-separated values:

```rust
use rtoon::encode_default;
use serde_json::json;

let data = json!({ "tags": ["admin", "ops", "dev"] });
println!("{}", encode_default(&data).unwrap());
```

**Output:**

```toon
tags[3]: admin,ops,dev
```

### Arrays of Objects (Tabular)

When arrays contain uniform objects with the same keys and primitive-only values, they're encoded in tabular format for maximum token efficiency:

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

**Output:**

```toon
items[2]{sku,qty,price}:
  A1,2,9.99
  B2,1,14.5
```

Tabular arrays can be nested:

```rust
let nested = json!({
    "items": [
        {
            "users": [
                { "id": 1, "name": "Ada" },
                { "id": 2, "name": "Bob" }
            ],
            "status": "active"
        }
    ]
});
println!("{}", encode_default(&nested).unwrap());
```

**Output:**

```toon
items[1]:
  status: active
  users[2]{id,name}:
    1,Ada
    2,Bob
```

### Arrays of Arrays

When arrays contain other primitive arrays, they're expanded as list items:

```rust
use rtoon::encode_default;
use serde_json::json;

let data = json!({
    "pairs": [[1, 2], [3, 4]]
});
println!("{}", encode_default(&data).unwrap());
```

**Output:**

```toon
pairs[2]:
  - [2]: 1,2
  - [2]: 3,4
```

### Mixed Arrays

Non-uniform arrays (containing primitives, objects, or nested arrays) use the expanded list format:

```rust
use rtoon::encode_default;
use serde_json::json;

let mixed = json!({
    "items": [1, {"a": 1}, "text"]
});
println!("{}", encode_default(&mixed).unwrap());
```

**Output:**

```toon
items[3]:
  - 1
  - a: 1
  - text
```

Objects in list format place the first field on the hyphen line:

```rust
let list_objects = json!({
    "items": [
        {"id": 1, "name": "First"},
        {"id": 2, "name": "Second", "extra": true}
    ]
});
println!("{}", encode_default(&list_objects).unwrap());
```

**Output:**

```toon
items[2]:
  - id: 1
    name: First
  - id: 2
    name: Second
    extra: true
```

### Custom Delimiters

Use tab or pipe delimiters to avoid quoting and save more tokens:

```rust
use rtoon::{encode, EncodeOptions, Delimiter};
use serde_json::json;

let data = json!({
    "items": [
        {"sku": "A1", "name": "Widget", "qty": 2, "price": 9.99},
        {"sku": "B2", "name": "Gadget", "qty": 1, "price": 14.5}
    ]
});

// Tab delimiter (\t)
let tab = encode(&data, &EncodeOptions::new().with_delimiter(Delimiter::Tab)).unwrap();
println!("{}", tab);

// Pipe delimiter (|)
let pipe = encode(&data, &EncodeOptions::new().with_delimiter(Delimiter::Pipe)).unwrap();
println!("{}", pipe);
```

### Length Markers

Prefix array lengths with a marker character for clarity:

```rust
use rtoon::{encode, EncodeOptions};
use serde_json::json;

let data = json!({
    "tags": ["reading", "gaming", "coding"],
    "items": [
        {"sku": "A1", "qty": 2, "price": 9.99},
        {"sku": "B2", "qty": 1, "price": 14.5}
    ]
});

let opts = EncodeOptions::new().with_length_marker('#');
println!("{}", encode(&data, &opts).unwrap());
```

**Output:**

```toon
items[#2]{sku,qty,price}:
  A1,2,9.99
  B2,1,14.5
tags[#3]: reading,gaming,coding
```

### Empty Containers & Root Forms

Empty arrays and objects are supported:

```rust
use rtoon::encode_default;
use serde_json::json;

// Empty array
let empty_items = json!({ "items": [] });
println!("{}", encode_default(&empty_items).unwrap());

// Root array
let root_array = json!(["x", "y"]);
println!("{}", encode_default(&root_array).unwrap());
```

**Output:**

```toon
items[0]:

[2]: x,y
```

Empty objects at root encode to empty output.

### Round-Trip Encoding

TOON supports full round-trip encoding and decoding:

```rust
use rtoon::{decode_default, encode_default};
use serde_json::json;

let original = json!({
    "product": "Widget",
    "price": 29.99,
    "stock": 100,
    "categories": ["tools", "hardware"]
});

let encoded = encode_default(&original).unwrap();
let decoded = decode_default(&encoded).unwrap();

assert_eq!(original, decoded);
println!("Round-trip successful!");
```

### Strict Mode Decoding

Strict mode enforces array counts, indentation, and delimiter consistency:

```rust
use rtoon::{decode, DecodeOptions};

// Malformed: header says 2 rows, but only 1 provided
let malformed = "items[2]{id,name}:\n  1,Ada";

let opts = DecodeOptions::new().with_strict(true);
match decode(malformed, &opts) {
    Ok(_) => println!("Unexpectedly decoded"),
    Err(err) => println!("Strict decode error: {}", err),
}
```

Strict mode (default) checks:
- Array counts must match declared lengths
- Indentation must be exact multiples of indent size
- Tabs cannot be used for indentation
- Invalid escape sequences cause errors
- Missing colons after keys cause errors
- Blank lines inside arrays/tabular rows cause errors

---

## API Reference

### Encoding Functions

```rust
pub fn encode(value: &serde_json::Value, options: &EncodeOptions) -> ToonResult<String>
pub fn encode_default(value: &serde_json::Value) -> ToonResult<String>
```

### Decoding Functions

```rust
pub fn decode(input: &str, options: &DecodeOptions) -> ToonResult<serde_json::Value>
pub fn decode_default(input: &str) -> ToonResult<serde_json::Value>
pub fn decode_strict(input: &str) -> ToonResult<serde_json::Value>
```

### EncodeOptions

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeOptions {
    pub delimiter: Delimiter,         // default: Delimiter::Comma
    pub length_marker: Option<char>,   // default: None
    pub indent: String,               // default: "  " (2 spaces)
}

impl EncodeOptions {
    pub fn new() -> Self
    pub fn with_delimiter(self, delimiter: Delimiter) -> Self
    pub fn with_length_marker(self, marker: char) -> Self
    pub fn with_indent(self, indent: impl Into<String>) -> Self
}
```

**Example:**

```rust
use rtoon::{encode, EncodeOptions, Delimiter};

let opts = EncodeOptions::new()
    .with_delimiter(Delimiter::Tab)
    .with_length_marker('#')
    .with_indent("    "); // 4 spaces
```

### DecodeOptions

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeOptions {
    pub delimiter: Option<Delimiter>,  // auto-detect if None
    pub strict: bool,                 // default: true
}

impl DecodeOptions {
    pub fn new() -> Self
    pub fn with_strict(self, strict: bool) -> Self
    pub fn with_delimiter(self, delimiter: Delimiter) -> Self
}
```

**Example:**

```rust
use rtoon::{decode, DecodeOptions, Delimiter};

let opts = DecodeOptions::new()
    .with_strict(true)
    .with_delimiter(Some(Delimiter::Pipe));
```

### Delimiter

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Comma,  // ","
    Tab,    // "\t" (U+0009)
    Pipe,   // "|"
}
```

### Error Handling

All functions return `ToonResult<T>`, which is `Result<T, ToonError>`. The error type provides detailed information about parsing or encoding failures:

```rust
use rtoon::{decode_default, ToonError};

match decode_default(input) {
    Ok(value) => println!("Success: {}", value),
    Err(ToonError::ParseError(msg)) => eprintln!("Parse error: {}", msg),
    Err(ToonError::ValidationError(msg)) => eprintln!("Validation error: {}", msg),
    // ... other error variants
}
```

---

## Format Overview

- **Objects:** `key: value` with 2-space indentation for nesting
- **Primitive arrays:** inline with count, e.g., `tags[3]: a,b,c`
- **Arrays of objects:** tabular header, e.g., `items[2]{id,name}:\n  ...`
- **Mixed arrays:** list format with `- ` prefix
- **Quoting:** only when necessary (special chars, ambiguity, keywords like `true`, `null`)
- **Root forms:** objects (default), arrays, or primitives

For complete format specification, see [SPEC.md](./SPEC.md).

## Specification

This implementation follows the [TOON Specification v1.2](./SPEC.md). The specification defines:

- Data model and normalization rules
- Encoding and decoding semantics
- Header syntax and delimiter scoping
- Quoting rules and escaping
- Strict mode validation requirements

Refer to [SPEC.md](./SPEC.md) for complete details.

## Running Examples

Run the consolidated examples:

```bash
cargo run --example examples
```

This executes `examples/main.rs`, which invokes all parts under `examples/parts/`:

- `arrays.rs` — Primitive array encoding
- `arrays_of_arrays.rs` — Nested primitive arrays
- `objects.rs` — Simple and nested objects
- `tabular.rs` — Tabular array encoding
- `delimiters.rs` — Custom delimiter usage
- `mixed_arrays.rs` — Mixed/non-uniform arrays
- `length_marker.rs` — Length marker examples
- `empty_and_root.rs` — Edge cases and root forms
- `round_trip.rs` — Encoding and decoding verification
- `decode_strict.rs` — Strict mode validation

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

<details>
<summary><strong>🤝 How to Contribute</strong></summary>

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

</details>

## License

MIT © 2025

## See Also

- **Original JavaScript/TypeScript implementation:** [@byjohann/toon](https://github.com/johannschopplich/toon)
---

<div align="center">

**Built with ❤️ in Rust**

</div>
