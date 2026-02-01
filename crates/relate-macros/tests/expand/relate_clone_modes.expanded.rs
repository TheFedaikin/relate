//! Clone modes expansion test - shows how cloned/move/copy affect generated code.
use relate::Relate;
struct Source {
    name: String,
    value: i32,
}
#[relate(Source, cloned)]
struct Target {
    name: String,
    value: i32,
}
impl ::core::convert::From<Source> for Target {
    fn from(src: Source) -> Self {
        Self {
            name: src.name.clone(),
            value: src.value.clone(),
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
