//! Should fail: #[derive(Relate)] without #[relate(...)] attribute.

use relate::Relate;

struct Source {
    name: String,
}

#[derive(Relate)]
struct Target {
    name: String,
}

fn main() {}

