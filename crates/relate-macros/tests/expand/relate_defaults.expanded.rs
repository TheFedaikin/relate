//! Defaults expansion test - shows how default values are generated.
use relate::Relate;
struct Source {
    name: String,
}
#[relate(Source)]
struct Target {
    name: String,
    #[relate(default)]
    active: bool,
    #[relate(default = 42)]
    count: i32,
}
impl ::core::convert::From<Source> for Target {
    fn from(src: Source) -> Self {
        Self {
            name: src.name,
            active: ::core::default::Default::default(),
            count: 42,
        }
    }
}
impl ::core::convert::From<&Source> for Target {
    fn from(src: &Source) -> Self {
        Self {
            name: src.name.clone(),
            active: ::core::default::Default::default(),
            count: 42,
        }
    }
}
fn main() {}
