//! Relate derive with transforms expansion test.

use relate::Relate;

struct Source {
    id: Option<i32>,
    name: String,
    encrypted: bool,
    value: String,
}

#[derive(Relate)]
#[relate(Source)]
struct Target {
    #[relate(_.unwrap_or(0))]
    id: i32,
    name: String,
    #[relate(default = None)]
    extra: Option<String>,
    #[relate(with = if .encrypted { None } else { Some(.value.clone()) })]
    value: Option<String>,
    encrypted: bool,
}

fn main() {}
