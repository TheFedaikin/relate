//! Should fail: target field not mapped and no default.

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
        // Missing: value
    }
}

fn main() {}
