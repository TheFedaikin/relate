//! relate_structs! with defaults - should compile.

use relate::relate_structs;

#[derive(Debug, Clone)]
struct Source {
    name: String,
}

#[derive(Debug, Clone, Default)]
struct Target {
    name:  String,
    count: i32,
    label: String,
}

relate_structs! {
    Source ~> Target {
        name;
        count: default = 0;
        label: default;
    }
}

fn main() {
    let source = Source {
        name: "test".to_string(),
    };
    let target: Target = source.into();
    assert_eq!(target.name, "test");
    assert_eq!(target.count, 0);
    assert_eq!(target.label, "");
}
