//! Tests for field-level clone modifiers in relate_structs!

use relate::relate_structs;

// Test basic clone mode modifiers
mod basic_clone_modes {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        id:   i32,
        name: String,
        data: Vec<u8>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        id:   i32,
        name: String,
        data: Vec<u8>,
    }

    // Test `: cloned` modifier forces cloning
    relate_structs! {
        Source ~> Target {
            id;
            name: cloned;
            data;
        }
    }

    #[test]
    fn test_cloned_modifier() {
        let source = Source {
            id:   42,
            name: "test".to_string(),
            data: vec![1, 2, 3],
        };
        let target: Target = source.into();
        assert_eq!(target.id, 42);
        assert_eq!(target.name, "test");
        assert_eq!(target.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_from_ref() {
        let source = Source {
            id:   42,
            name: "test".to_string(),
            data: vec![1, 2, 3],
        };
        let target: Target = (&source).into();
        assert_eq!(target.id, 42);
        assert_eq!(target.name, "test");
        // source still accessible
        assert_eq!(source.id, 42);
    }
}

// Test `: copy` modifier with Copy types
mod copy_modifier {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct CopySource {
        x: i32,
        y: i32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct CopyTarget {
        x: i32,
        y: i32,
    }

    // `: copy` asserts the type is Copy (no clone needed)
    relate_structs! {
        CopySource ~> CopyTarget {
            x: copy;
            y: copy;
        }
    }

    #[test]
    fn test_copy_modifier() {
        let source = CopySource { x: 10, y: 20 };
        let target: CopyTarget = source.into();
        assert_eq!(target.x, 10);
        assert_eq!(target.y, 20);
    }

    #[test]
    fn test_copy_from_ref() {
        let source = CopySource { x: 10, y: 20 };
        let target: CopyTarget = (&source).into();
        assert_eq!(target.x, 10);
        assert_eq!(target.y, 20);
        // source still accessible
        assert_eq!(source.x, 10);
    }
}

// Test `: move` modifier forces move semantics
mod move_modifier {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        name: String,
        data: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        name: String,
        data: String,
    }

    // `: move` hints that field should be moved (no clone in owned impl)
    relate_structs! {
        Source ~> Target {
            name: move;
            data;
        }
    }

    #[test]
    fn test_move_modifier_owned() {
        let source = Source {
            name: "test".to_string(),
            data: "data".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.name, "test");
        assert_eq!(target.data, "data");
    }

    #[test]
    fn test_move_modifier_ref() {
        // From ref still clones (move only affects owned impl)
        let source = Source {
            name: "test".to_string(),
            data: "data".to_string(),
        };
        let target: Target = (&source).into();
        assert_eq!(target.name, "test");
        assert_eq!(target.data, "data");
        // source still accessible since it was borrowed
        assert_eq!(source.name, "test");
    }
}

// Test clone modifiers with transforms
mod clone_modes_with_transform {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        value: String,
    }

    // Clone modifier combined with method transform
    relate_structs! {
        Source ~> Target {
            value: with = _.clone().unwrap_or_default(), cloned;
        }
    }

    #[test]
    fn test_transform_with_clone_mode() {
        let source = Source {
            value: Some("hello".to_string()),
        };
        let target: Target = source.into();
        assert_eq!(target.value, "hello");
    }

    #[test]
    fn test_transform_with_clone_mode_none() {
        let source = Source { value: None };
        let target: Target = source.into();
        assert_eq!(target.value, "");
    }
}

// Test clone modifiers with field access from different source field
mod clone_modes_with_field_access {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        source_name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        target_name: String,
    }

    // Clone modifier with field access from different source field
    relate_structs! {
        Source ~> Target {
            target_name: with = .source_name.clone(), cloned;
        }
    }

    #[test]
    fn test_field_access_with_clone_mode() {
        let source = Source {
            source_name: "test".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.target_name, "test");
    }
}
