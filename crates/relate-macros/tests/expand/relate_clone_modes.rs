//! Clone modes expansion test - shows how cloned/move/copy affect generated code.

use relate::Relate;

struct Source {
    name: String,
    value: i32,
}

#[derive(Relate)]
#[relate(Source, cloned)]
struct Target {
    name: String,
    value: i32,
}

fn main() {}
