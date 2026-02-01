//! Tests for Unicode identifiers in struct fields.

use relate::Relate;

#[derive(Debug, Clone)]
struct SourceWithUnicode {
    // Cyrillic
    имя: String,
    // Greek
    τιμή: i32,
    // Chinese
    值: bool,
    // Japanese hiragana
    なまえ: String,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(SourceWithUnicode)]
struct TargetWithUnicode {
    имя:   String,
    τιμή:  i32,
    值:     bool,
    なまえ: String,
}

fn main() {
    let source = SourceWithUnicode {
        имя:   "Привет".to_string(),
        τιμή:  42,
        值:     true,
        なまえ: "こんにちは".to_string(),
    };

    let target: TargetWithUnicode = source.into();

    assert_eq!(target.имя, "Привет");
    assert_eq!(target.τιμή, 42);
    assert!(target.值);
    assert_eq!(target.なまえ, "こんにちは");
}
