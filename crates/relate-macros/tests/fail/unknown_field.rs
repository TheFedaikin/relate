//! Should fail: referencing a field that doesn't exist in target.

use relate::relate_structs;

#[derive(Debug, Clone)]
struct Source {
    name: String,
}

#[derive(Debug, Clone)]
struct Target {
    name: String,
}

relate_structs! {
    Source ~> Target {
        name;
        nonexistent_field;
    }
}

fn main() {}
