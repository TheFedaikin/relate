//! Relate derive with bidirectional conversion expansion test.

use relate::Relate;

struct Barcodes {
    ean13: Option<String>,
    ean8:  Option<String>,
    code:  Option<String>,
}

#[derive(Relate)]
#[relate(Barcodes, both)]
struct DbBarcodes {
    ean13: Option<String>,
    ean8:  Option<String>,
    code:  Option<String>,
}

fn main() {}

