use relate::relate_structs;

struct Source {
    value: String,
}

struct Target {
    value: i32,
}

// Error: Using ~>? but .parse() without ? won't propagate errors correctly
// This will compile but the Result type won't match
relate_structs! {
    Source ~>? Target {
        value: with = _.parse();
    }
}

fn main() {}
