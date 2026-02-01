//! Should fail: bidirectional with transform that can't reverse.

use relate::relate_structs;

#[derive(Debug, Clone)]
struct Source {
    name:  String,
    value: i32,
}

#[derive(Debug, Clone)]
struct Target {
    name:    String,
    doubled: i32,
}

// This should fail because `with = .value * 2` can't be reversed automatically
relate_structs! {
    Source ~ Target {
        name;
        doubled: with = .value * 2;
    }
}

fn main() {}
