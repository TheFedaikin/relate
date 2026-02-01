//! Relate derive with field rename expansion test.

use relate::Relate;

struct Source {
    id:   String,
    name: String,
}

#[derive(Relate)]
#[relate(Source)]
struct Target {
    #[relate(.id)]
    moysklad_id: String,
    name:        String,
    #[relate(.name)]
    sync_name:   String,
}

fn main() {}

