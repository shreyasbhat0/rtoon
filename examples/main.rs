#[path = "parts/arrays.rs"] mod arrays;
#[path = "parts/arrays_of_arrays.rs"] mod arrays_of_arrays;
#[path = "parts/objects.rs"] mod objects;
#[path = "parts/delimiters.rs"] mod delimiters;
#[path = "parts/mixed_arrays.rs"] mod mixed_arrays;
#[path = "parts/round_trip.rs"] mod round_trip;
#[path = "parts/tabular.rs"] mod tabular;
#[path = "parts/length_marker.rs"] mod length_marker;
#[path = "parts/empty_and_root.rs"] mod empty_and_root;
#[path = "parts/decode_strict.rs"] mod decode_strict;

fn main(){
    println!("=== R-Toon Consolidated Examples ===\n");

    println!("-- array examples --");
    arrays::arrays();

    println!("\n-- arrays of arrays --");
    arrays_of_arrays::arrays_of_arrays();

    println!("\n-- objects --");
    objects::objects();

    println!("\n-- delimiters --");
    delimiters::delimiters();

    println!("\n-- mixed arrays --");
    mixed_arrays::mixed_arrays();

    println!("\n-- round trip --");
    round_trip::round_trip();

    println!("\n-- tabular --");
    tabular::tabular();

    println!("\n-- length marker --");
    length_marker::length_marker();

    println!("\n-- empty and root --");
    empty_and_root::empty_and_root();

    println!("\n-- decode strict --");
    decode_strict::decode_strict();

    println!("\n=== Examples Complete ===");
}