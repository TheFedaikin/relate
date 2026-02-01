//! Tests for bidirectional Relate derive.

use relate::Relate;

#[derive(Debug, Clone, PartialEq)]
struct TypeA {
    name:  String,
    value: i32,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(TypeA, both)]
struct TypeB {
    name:  String,
    value: i32,
}

#[test]
fn test_bidirectional_forward() {
    let a = TypeA {
        name:  "test".to_string(),
        value: 42,
    };

    let b: TypeB = a.into();

    assert_eq!(b.name, "test");
    assert_eq!(b.value, 42);
}

#[test]
fn test_bidirectional_backward() {
    let b = TypeB {
        name:  "test".to_string(),
        value: 42,
    };

    let a: TypeA = b.into();

    assert_eq!(a.name, "test");
    assert_eq!(a.value, 42);
}

#[test]
fn test_bidirectional_from_ref() {
    let a = TypeA {
        name:  "ref".to_string(),
        value: 100,
    };

    // Forward from ref
    let b: TypeB = (&a).into();
    assert_eq!(b.name, "ref");

    // a still usable
    assert_eq!(a.name, "ref");

    // Backward from ref
    let a2: TypeA = (&b).into();
    assert_eq!(a2.name, "ref");

    // b still usable
    assert_eq!(b.name, "ref");
}

#[test]
fn test_roundtrip() {
    let original = TypeA {
        name:  "roundtrip".to_string(),
        value: 123,
    };

    let b: TypeB = original.clone().into();
    let back: TypeA = b.into();

    assert_eq!(original, back);
}

// Test bidirectional with same field types (like Barcodes <-> DbBarcodes)
#[derive(Debug, Clone, PartialEq)]
struct Barcodes {
    ean13:   Option<String>,
    ean8:    Option<String>,
    code128: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(Barcodes, both)]
struct DbBarcodes {
    ean13:   Option<String>,
    ean8:    Option<String>,
    code128: Option<String>,
}

#[test]
fn test_barcodes_bidirectional() {
    let barcodes = Barcodes {
        ean13:   Some("1234567890123".to_string()),
        ean8:    None,
        code128: Some("CODE".to_string()),
    };

    let db: DbBarcodes = barcodes.clone().into();
    assert_eq!(db.ean13, Some("1234567890123".to_string()));

    let back: Barcodes = db.into();
    assert_eq!(barcodes, back);
}
