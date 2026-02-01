//! Basic relate_structs! usage - should compile.

use relate::relate_structs;

#[derive(Debug, Clone)]
struct Source {
    name:  String,
    value: i32,
}

#[derive(Debug, Clone)]
struct Target {
    name:  String,
    value: i32,
}

relate_structs! {
    Source ~> Target {
        name;
        value;
    }
}

fn main() {
    let source = Source {
        name:  "test".to_string(),
        value: 42,
    };
    let _target: Target = source.into();
}
