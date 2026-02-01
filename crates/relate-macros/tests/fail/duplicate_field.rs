use relate::relate_structs;

struct Source {
    id:   i32,
    name: String,
}

struct Target {
    id:   i32,
    name: String,
}

// Error: field `id` specified twice
relate_structs! {
    Source ~> Target {
        id;
        name;
        id;
    }
}

fn main() {}
