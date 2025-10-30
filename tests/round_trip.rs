use serde_json::json;
use rtoon::{decode_default, encode_default};

#[test]
fn test_comprehensive_round_trips() {
    let test_cases = vec![
        json!(null),
        json!(true),
        json!(false),
        json!(42),
        json!(-42),
        json!(3.14),
        json!("hello"),
        json!(""),
        json!({"key": "value"}),
        json!({"a": 1, "b": 2, "c": 3}),
        json!({"nested": {"key": "value"}}),
        json!({"array": [1, 2, 3]}),
        json!({"mixed": [1, "two", true, null]}),
        json!({"users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]}),
        json!({"empty_array": []}),
        json!({"empty_object": {}}),
    ];

    for (i, case) in test_cases.iter().enumerate() {
        let encoded = encode_default(case)
            .unwrap_or_else(|e| panic!("Failed to encode case {}: {:?}", i, e));
        let decoded = decode_default(&encoded)
            .unwrap_or_else(|e| panic!("Failed to decode case {}: {:?}", i, e));
        assert_eq!(case, &decoded,
            "Round-trip failed for case {}: Original: {:?}, Decoded: {:?}", i, case, decoded);
    }
}
