//! Tests for large structs to verify clone optimization at scale.

use relate::Relate;

/// Large struct with 25 fields - tests clone optimization behavior
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct LargeSource {
    field01: String,
    field02: String,
    field03: String,
    field04: String,
    field05: String,
    field06: i32,
    field07: i32,
    field08: i32,
    field09: i32,
    field10: i32,
    field11: String,
    field12: String,
    field13: String,
    field14: String,
    field15: String,
    field16: bool,
    field17: bool,
    field18: bool,
    field19: bool,
    field20: bool,
    field21: f64,
    field22: f64,
    field23: f64,
    field24: f64,
    field25: f64,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(LargeSource)]
struct LargeTarget {
    field01: String,
    field02: String,
    field03: String,
    field04: String,
    field05: String,
    field06: i32,
    field07: i32,
    field08: i32,
    field09: i32,
    field10: i32,
    field11: String,
    field12: String,
    field13: String,
    field14: String,
    field15: String,
    field16: bool,
    field17: bool,
    field18: bool,
    field19: bool,
    field20: bool,
    field21: f64,
    field22: f64,
    field23: f64,
    field24: f64,
    field25: f64,
}

#[test]
fn test_large_struct_conversion() {
    let source = LargeSource {
        field01: "a".to_string(),
        field02: "b".to_string(),
        field03: "c".to_string(),
        field04: "d".to_string(),
        field05: "e".to_string(),
        field06: 1,
        field07: 2,
        field08: 3,
        field09: 4,
        field10: 5,
        field11: "f".to_string(),
        field12: "g".to_string(),
        field13: "h".to_string(),
        field14: "i".to_string(),
        field15: "j".to_string(),
        field16: true,
        field17: false,
        field18: true,
        field19: false,
        field20: true,
        field21: 1.1,
        field22: 2.2,
        field23: 3.3,
        field24: 4.4,
        field25: 5.5,
    };

    let target: LargeTarget = source.into();

    assert_eq!(target.field01, "a");
    assert_eq!(target.field15, "j");
    assert_eq!(target.field25, 5.5);
}

#[test]
fn test_large_struct_from_ref() {
    let source = LargeSource {
        field01: "test".to_string(),
        field02: "data".to_string(),
        field03: "value".to_string(),
        field04: "more".to_string(),
        field05: "stuff".to_string(),
        field06: 10,
        field07: 20,
        field08: 30,
        field09: 40,
        field10: 50,
        field11: "x".to_string(),
        field12: "y".to_string(),
        field13: "z".to_string(),
        field14: "w".to_string(),
        field15: "v".to_string(),
        field16: false,
        field17: true,
        field18: false,
        field19: true,
        field20: false,
        field21: 0.1,
        field22: 0.2,
        field23: 0.3,
        field24: 0.4,
        field25: 0.5,
    };

    let target: LargeTarget = (&source).into();

    assert_eq!(target.field01, "test");
    assert_eq!(target.field10, 50);
    // Original still usable after ref conversion
    assert_eq!(source.field25, 0.5);
}

/// Test large struct with mixed transforms and defaults
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MixedSource {
    name:   String,
    value1: String,
    value2: String,
    value3: i32,
    value4: i32,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(MixedSource)]
struct MixedTarget {
    name:    String,
    #[relate(_.to_uppercase())]
    value1:  String,
    #[relate(.value2.len())]
    length:  usize,
    value3:  i32,
    value4:  i32,
    #[relate(default)]
    extra:   i32,
    #[relate(default = "generated".to_string())]
    gen_str: String,
}

#[test]
fn test_large_mixed_transforms() {
    let source = MixedSource {
        name:   "test".to_string(),
        value1: "hello".to_string(),
        value2: "world".to_string(),
        value3: 42,
        value4: 100,
    };

    let target: MixedTarget = source.into();

    assert_eq!(target.name, "test");
    assert_eq!(target.value1, "HELLO");
    assert_eq!(target.length, 5); // "world".len()
    assert_eq!(target.value3, 42);
    assert_eq!(target.value4, 100);
    assert_eq!(target.extra, 0);
    assert_eq!(target.gen_str, "generated");
}
