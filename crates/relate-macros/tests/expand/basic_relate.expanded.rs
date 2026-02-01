//! Basic Relate derive expansion test.
use relate::Relate;
struct Source {
    name: String,
    value: i32,
}
#[relate(Source)]
struct Target {
    name: String,
    value: i32,
}
impl ::core::convert::From<Source> for Target {
    fn from(src: Source) -> Self {
        Self {
            name: src.name,
            value: src.value,
        }
    }
}
impl ::core::convert::From<&Source> for Target {
    fn from(src: &Source) -> Self {
        Self {
            name: src.name.clone(),
            value: src.value.clone(),
        }
    }
}
fn main() {}
