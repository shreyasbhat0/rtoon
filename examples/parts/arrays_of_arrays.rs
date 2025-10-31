use rtoon::encode_default;
use serde_json::json;

pub fn arrays_of_arrays() {
    // Arrays containing primitive inner arrays
    let pairs = json!({
        "pairs": [[1, 2], [3, 4]]
    });
    let out = encode_default(&pairs).unwrap();
    println!("{}", out);
}
