//! #[derive(Relate)] with transforms - should compile.

use relate::Relate;

struct Source {
    id:    Option<i32>,
    name:  String,
    count: i32,
}

#[derive(Relate)]
#[relate(Source)]
struct Target {
    #[relate(_.unwrap_or(0))]
    id: i32,

    #[relate(_.to_uppercase())]
    name: String,

    #[relate(.count)]
    num: i32,
}

fn main() {
    let source = Source {
        id:    Some(42),
        name:  "hello".to_string(),
        count: 100,
    };
    let target: Target = source.into();
    assert_eq!(target.id, 42);
    assert_eq!(target.name, "HELLO");
    assert_eq!(target.num, 100);
}

