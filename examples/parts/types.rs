use rtoon::{
    ToonResult, from_toon, to_toon
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug,Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
}

pub fn example() ->  ToonResult<()>{
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    let toon = to_toon(&user,None)?;
    println!("{:?}",toon);

    let decoded: User = from_toon(&toon,None)?;
    println!("{:?}",decoded);
    Ok(())
}
