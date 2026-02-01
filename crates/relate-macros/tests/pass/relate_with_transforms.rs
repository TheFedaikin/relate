//! relate_structs! with transforms - should compile.

use relate::relate_structs;

#[derive(Debug, Clone)]
struct Source {
    id:    Option<i32>,
    name:  String,
    count: i32,
}

#[derive(Debug, Clone)]
struct Target {
    id:        i32,
    uppercase: String,
    doubled:   i32,
}

relate_structs! {
    Source ~> Target {
        id: with = _.unwrap_or(0);
        uppercase: with = .name.to_uppercase();
        doubled: with = .count * 2;
    }
}

fn main() {
    let source = Source {
        id:    Some(42),
        name:  "hello".to_string(),
        count: 21,
    };
    let target: Target = source.into();
    assert_eq!(target.id, 42);
    assert_eq!(target.uppercase, "HELLO");
    assert_eq!(target.doubled, 42);
}
