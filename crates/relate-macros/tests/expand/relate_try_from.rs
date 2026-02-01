//! TryFrom expansion test - shows how fallible expressions generate TryFrom.

use relate::Relate;

struct Source {
    port: String,
    host: String,
}

#[derive(Relate)]
#[relate(Source)]
struct Target {
    #[relate(_.parse()?)]
    port: u16,
    host: String,
}

fn main() {}
