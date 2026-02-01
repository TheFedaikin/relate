//! Collection mapping expansion test - shows how collection transforms are generated.
use relate::Relate;
struct Item {
    id: i32,
    name: String,
}
struct Source {
    items: Vec<Item>,
}
#[relate(Source)]
struct Target {
    #[relate([_.id])]
    items: Vec<i32>,
}
impl ::core::convert::From<Source> for Target {
    fn from(src: Source) -> Self {
        Self {
            items: src.items.iter().map(|__item| __item.id).collect(),
        }
    }
}
impl ::core::convert::From<&Source> for Target {
    fn from(src: &Source) -> Self {
        Self {
            items: src.items.iter().map(|__item| __item.id).collect(),
        }
    }
}
fn main() {}
