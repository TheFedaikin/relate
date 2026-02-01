//! Should fail: referencing a source field that doesn't exist.

use relate::relate_structs;

#[derive(Debug, Clone)]
struct Source {
    name: String,
}

#[derive(Debug, Clone)]
struct Target {
    name:  String,
    value: i32,
}

relate_structs! {
    Source ~> Target {
        name;
        value: with = .nonexistent;
    }
}

fn main() {}
