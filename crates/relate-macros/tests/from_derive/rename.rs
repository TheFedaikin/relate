//! Tests for field renaming in Relate derive.

use relate::Relate;

#[derive(Debug, Clone)]
struct Source {
    id:          String,
    name:        String,
    description: String,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(Source)]
struct Target {
    #[relate(.id)]
    moysklad_id: String,
    name:        String,
    #[relate(.description)]
    desc:        String,
}

#[test]
fn test_rename_field() {
    let source = Source {
        id:          "ms-123".to_string(),
        name:        "Test".to_string(),
        description: "A description".to_string(),
    };

    let target: Target = source.into();

    assert_eq!(target.moysklad_id, "ms-123");
    assert_eq!(target.name, "Test");
    assert_eq!(target.desc, "A description");
}

// Test rename with transform
#[derive(Debug, Clone)]
struct OptSource {
    id:   Option<i32>,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(OptSource)]
struct OptTarget {
    #[relate(.id.unwrap_or(0))]
    identifier: i32,
    name:       String,
}

#[test]
fn test_rename_with_transform() {
    let source = OptSource {
        id:   Some(42),
        name: "test".to_string(),
    };

    let target: OptTarget = source.into();

    assert_eq!(target.identifier, 42);
    assert_eq!(target.name, "test");
}
