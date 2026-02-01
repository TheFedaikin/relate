//! Test for invalid clone mode error message.

use relate::Relate;

#[derive(Debug, Clone)]
struct Source {
    value: String,
}

// Invalid clone mode
#[derive(Debug, Clone, Relate)]
#[relate(Source, invalid_mode)]
struct Target {
    value: String,
}

fn main() {}
