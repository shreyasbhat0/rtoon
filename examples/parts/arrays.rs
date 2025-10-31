use rtoon::encode_default;
use serde_json::json;

pub fn arrays() {
    let data = json!({ "tags": ["admin", "ops", "dev"] });
    let out = encode_default(&data).unwrap();
    println!("{}", out);
}
