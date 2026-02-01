//! Defaults expansion test - shows how default values are generated.

use relate::Relate;

struct Source {
    name: String,
}

#[derive(Relate)]
#[relate(Source)]
struct Target {
    name: String,
    #[relate(default)]
    active: bool,
    #[relate(default = 42)]
    count: i32,
}

fn main() {}
