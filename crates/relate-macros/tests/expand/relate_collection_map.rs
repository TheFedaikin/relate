//! Collection mapping expansion test - shows how collection transforms are generated.

use relate::Relate;

struct Item {
    id: i32,
    name: String,
}

struct Source {
    items: Vec<Item>,
}

#[derive(Relate)]
#[relate(Source)]
struct Target {
    #[relate([_.id])]
    items: Vec<i32>,
}

fn main() {}
