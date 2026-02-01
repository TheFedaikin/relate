//! Should fail: Unknown option in #[relate(...)] attribute

use relate::Relate;

struct Source {
    name: String,
}

#[derive(Relate)]
#[relate(Source, unknown)]
struct Target {
    name: String,
}

fn main() {}
