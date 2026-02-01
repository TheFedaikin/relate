//! Hygiene test: verifies macros work without implicit prelude.
//!
//! This test ensures all generated code uses fully qualified paths
//! and doesn't rely on items being in scope.

#![no_implicit_prelude]

extern crate relate;
extern crate std;

use relate::Relate;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::convert::{From, Into};
use std::fmt::Debug;
use std::string::String;

// Basic hygiene test for derive macro
#[derive(Debug, Clone, PartialEq)]
struct HygieneSource {
    name:  String,
    value: i32,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(HygieneSource)]
struct HygieneTarget {
    name:  String,
    value: i32,
}

// Hygiene test with default field
#[derive(Debug, Clone, PartialEq)]
struct SourceWithExtra {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(SourceWithExtra)]
struct TargetWithDefault {
    name:  String,
    #[relate(default)]
    extra: i32,
}

// Hygiene test with transform
#[derive(Debug, Clone, PartialEq)]
struct TransformSource {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(TransformSource)]
struct TransformTarget {
    #[relate(_.to_uppercase())]
    value: String,
}

fn main() {
    // Test basic conversion
    let source = HygieneSource {
        name:  String::from("test"),
        value: 42,
    };
    let target: HygieneTarget = source.into();
    assert!(target.name == String::from("test"));
    assert!(target.value == 42);

    // Test default field
    let source2 = SourceWithExtra {
        name: String::from("default_test"),
    };
    let target2: TargetWithDefault = source2.into();
    assert!(target2.name == String::from("default_test"));
    assert!(target2.extra == 0);

    // Test transform
    let source3 = TransformSource {
        value: String::from("hello"),
    };
    let target3: TransformTarget = source3.into();
    assert!(target3.value == String::from("HELLO"));
}
