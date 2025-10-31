use rtoon::encode_default;
use serde_json::json;

pub fn objects() {
    // Simple object
    let simple = json!({
        "id": 123,
        "name": "Ada",
        "active": true
    });
    let out = encode_default(&simple).unwrap();
    println!("{}", out);

    // Nested object
    let nested = json!({
        "user": { "id": 123, "name": "Ada" }
    });
    let out_nested = encode_default(&nested).unwrap();
    println!("\n{}", out_nested);
}
