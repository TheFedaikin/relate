//! Test for empty collection brackets error.

use relate::Relate;

#[derive(Debug, Clone)]
struct Source {
    items: Vec<i32>,
}

// Empty collection map brackets should error
#[derive(Debug, Clone, Relate)]
#[relate(Source)]
struct Target {
    #[relate([])]
    items: Vec<i32>,
}

fn main() {}
