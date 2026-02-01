//! Basic Relate derive expansion test.

use relate::Relate;

struct Source {
    name:  String,
    value: i32,
}

#[derive(Relate)]
#[relate(Source)]
struct Target {
    name:  String,
    value: i32,
}

fn main() {}
