//! Macro expansion tests.
//!
//! These tests verify that the macros expand to the expected code.
//! Run with `cargo test` to verify expansions match snapshots.
//! Run with `MACROTEST=overwrite cargo test` to update snapshots.

#[test]
fn expand_relate_derive() { macrotest::expand("tests/expand/*.rs"); }
