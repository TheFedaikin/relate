use relate::relate_structs;

struct Source {
    name: String,
}

struct Target {
    name: String,
    id:   i32,
}

// Error: `default` requires `=` for expression, not `:`
relate_structs! {
    Source ~> Target {
        name;
        id: default: 42;
    }
}

fn main() {}
