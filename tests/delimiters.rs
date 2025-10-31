use rtoon::{
    decode_default,
    encode,
    encode_default,
    Delimiter,
    EncodeOptions,
};
use serde_json::json;

#[test]
fn test_delimiter_variants() {
    let data = json!({"tags": ["a", "b", "c"]});

    let encoded = encode_default(&data).unwrap();
    assert!(encoded.contains("a,b,c"));
    let decoded = decode_default(&encoded).unwrap();
    assert_eq!(data, decoded);

    let opts = EncodeOptions::new().with_delimiter(Delimiter::Pipe);
    let encoded = encode(&data, &opts).unwrap();
    assert!(encoded.contains("a|b|c"));
    let decoded = decode_default(&encoded).unwrap();
    assert_eq!(data, decoded);

    let opts = EncodeOptions::new().with_delimiter(Delimiter::Tab);
    let encoded = encode(&data, &opts).unwrap();
    assert!(encoded.contains("a\tb\tc"));
    let decoded = decode_default(&encoded).unwrap();
    assert_eq!(data, decoded);
}

#[test]
fn test_length_markers() {
    let data = json!({"items": [1, 2, 3, 4, 5]});

    let opts = EncodeOptions::new().with_length_marker('#');
    let encoded = encode(&data, &opts).unwrap();
    assert!(encoded.contains("#5"));
    let decoded = decode_default(&encoded).unwrap();
    assert_eq!(data, decoded);

    let encoded = encode_default(&data).unwrap();
    assert!(encoded.contains("[5]"));
    let decoded = decode_default(&encoded).unwrap();
    assert_eq!(data, decoded);
}

#[test]
fn test_delimiter_in_values() {
    let data = json!({"tags": ["a,b", "c|d", "e\tf"]});

    let encoded = encode_default(&data).unwrap();
    assert!(encoded.contains("\"a,b\""));
    let decoded = decode_default(&encoded).unwrap();
    assert_eq!(data, decoded);

    let opts = EncodeOptions::new().with_delimiter(Delimiter::Pipe);
    let encoded = encode(&data, &opts).unwrap();
    assert!(encoded.contains("\"c|d\""));
    let decoded = decode_default(&encoded).unwrap();
    assert_eq!(data, decoded);
}
