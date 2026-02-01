//! Basic #[derive(Relate)] usage - should compile.

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

fn main() {
    let source = Source {
        name:  "test".to_string(),
        value: 42,
    };
    let _target: Target = source.into();
}

