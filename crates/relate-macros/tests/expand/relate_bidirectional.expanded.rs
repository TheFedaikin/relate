//! Relate derive with bidirectional conversion expansion test.
use relate::Relate;
struct Barcodes {
    ean13: Option<String>,
    ean8: Option<String>,
    code: Option<String>,
}
#[relate(Barcodes, both)]
struct DbBarcodes {
    ean13: Option<String>,
    ean8: Option<String>,
    code: Option<String>,
}
impl ::core::convert::From<Barcodes> for DbBarcodes {
    fn from(src: Barcodes) -> Self {
        Self {
            ean13: src.ean13,
            ean8: src.ean8,
            code: src.code,
        }
    }
}
impl ::core::convert::From<&Barcodes> for DbBarcodes {
    fn from(src: &Barcodes) -> Self {
        Self {
            ean13: src.ean13.clone(),
            ean8: src.ean8.clone(),
            code: src.code.clone(),
        }
    }
}
impl ::core::convert::From<DbBarcodes> for Barcodes {
    fn from(src: DbBarcodes) -> Self {
        Self {
            ean13: src.ean13,
            ean8: src.ean8,
            code: src.code,
        }
    }
}
impl ::core::convert::From<&DbBarcodes> for Barcodes {
    fn from(src: &DbBarcodes) -> Self {
        Self {
            ean13: src.ean13.clone(),
            ean8: src.ean8.clone(),
            code: src.code.clone(),
        }
    }
}
fn main() {}
