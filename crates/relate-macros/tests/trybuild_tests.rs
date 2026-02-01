//! Trybuild tests for compile-time error messages.

#[test]
fn pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
}

#[test]
fn fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*.rs");
}
