//! Hygiene test for relate_structs! macro.
//!
//! Verifies the macro works without implicit prelude.

#![no_implicit_prelude]

extern crate relate;
extern crate std;

use relate::relate_structs;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::convert::{From, Into};
use std::fmt::Debug;
use std::string::String;

#[derive(Debug, Clone, PartialEq)]
struct HygieneA {
    name:  String,
    value: i32,
}

#[derive(Debug, Clone, PartialEq)]
struct HygieneB {
    name:  String,
    value: i32,
}

relate_structs! {
    HygieneA ~> HygieneB {
        name;
        value;
    }
}

// Test with default
#[derive(Debug, Clone, PartialEq)]
struct SourceC {
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
struct TargetC {
    name:  String,
    extra: i32,
}

relate_structs! {
    SourceC ~> TargetC {
        name;
        extra: default;
    }
}

fn main() {
    // Test basic conversion
    let a = HygieneA {
        name:  String::from("test"),
        value: 42,
    };
    let b: HygieneB = a.into();
    assert!(b.name == String::from("test"));
    assert!(b.value == 42);

    // Test with default
    let c = SourceC {
        name: String::from("test"),
    };
    let d: TargetC = c.into();
    assert!(d.name == String::from("test"));
    assert!(d.extra == 0);
}
