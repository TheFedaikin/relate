//! Tests for clone modes: `cloned`, `move`, and `auto` (default).

use relate::Relate;

// =============================================================================
// Struct-Level Clone Modes
// =============================================================================

mod struct_level_cloned {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        name:  String,
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct TargetCloned {
        name:  String,
        value: i32,
    }

    #[test]
    fn test_struct_level_cloned_owned() {
        let source = Source {
            name:  "test".to_string(),
            value: 42,
        };

        let target: TargetCloned = source.into();

        assert_eq!(target.name, "test");
        assert_eq!(target.value, 42);
    }

    #[test]
    fn test_struct_level_cloned_ref() {
        let source = Source {
            name:  "test".to_string(),
            value: 42,
        };

        let target: TargetCloned = (&source).into();

        assert_eq!(target.name, "test");
        // source still available
        assert_eq!(source.name, "test");
    }
}

mod struct_level_move {
    use super::*;

    #[derive(Debug)]
    struct Source {
        value: i32,
    }

    // Note: move mode means fields are moved, not cloned
    // This works for Copy types
    #[derive(Debug, PartialEq, Relate)]
    #[relate(Source, move)]
    struct TargetMove {
        value: i32,
    }

    #[test]
    fn test_struct_level_move_copy_types() {
        let source = Source { value: 42 };

        let target: TargetMove = source.into();

        assert_eq!(target.value, 42);
    }
}

// =============================================================================
// Field-Level Clone Mode Overrides
// =============================================================================

mod field_level_overrides {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        cloned_field: String,
        moved_field:  String,
        auto_field:   String,
    }

    // Default auto mode with field-level overrides
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct TargetFieldOverrides {
        #[relate(cloned)]
        cloned_field: String,
        // moved_field uses auto (default)
        moved_field:  String,
        auto_field:   String,
    }

    #[test]
    fn test_field_level_cloned_override() {
        let source = Source {
            cloned_field: "cloned".to_string(),
            moved_field:  "moved".to_string(),
            auto_field:   "auto".to_string(),
        };

        let target: TargetFieldOverrides = source.into();

        assert_eq!(target.cloned_field, "cloned");
        assert_eq!(target.moved_field, "moved");
        assert_eq!(target.auto_field, "auto");
    }

    // Struct cloned with field move override
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct ClonedWithMoveField {
        cloned_field: String,
        #[relate(move)]
        moved_field:  String,
    }

    #[test]
    fn test_cloned_struct_with_move_field() {
        let source = Source {
            cloned_field: "will clone".to_string(),
            moved_field:  "will move".to_string(),
            auto_field:   "unused".to_string(),
        };

        let target: ClonedWithMoveField = source.into();

        assert_eq!(target.cloned_field, "will clone");
        assert_eq!(target.moved_field, "will move");
    }
}

// =============================================================================
// Clone Mode with Renames
// =============================================================================

mod with_renames {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        original_name: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct RenamedCloned {
        #[relate(.original_name, cloned)]
        new_name: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct RenamedMove {
        #[relate(.original_name, move)]
        new_name: String,
    }

    #[test]
    fn test_rename_with_cloned() {
        let source = Source {
            original_name: "test".to_string(),
        };

        let target: RenamedCloned = source.into();
        assert_eq!(target.new_name, "test");
    }

    #[test]
    fn test_rename_with_move() {
        let source = Source {
            original_name: "test".to_string(),
        };

        let target: RenamedMove = source.into();
        assert_eq!(target.new_name, "test");
    }
}

// =============================================================================
// Clone Mode with Transforms
// =============================================================================

mod with_transforms {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        name: String,
    }

    // Method transform with clone mode
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct TransformCloned {
        #[relate(_.to_uppercase(), cloned)]
        name: String,
    }

    #[test]
    fn test_method_transform_with_cloned() {
        let source = Source {
            name: "hello".to_string(),
        };

        let target: TransformCloned = source.into();
        assert_eq!(target.name, "HELLO");
    }

    // with = expr with clone mode
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct WithExprCloned {
        #[relate(with = _.len(), cloned)]
        name: usize,
    }

    #[test]
    fn test_with_expr_with_cloned() {
        let source = Source {
            name: "hello".to_string(),
        };

        let target: WithExprCloned = source.into();
        assert_eq!(target.name, 5);
    }
}

// =============================================================================
// Multiple Fields Using Same Source (Auto Clone Detection)
// =============================================================================

mod auto_clone_detection {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        shared: String,
    }

    // Auto mode should detect that `shared` is used twice and clone appropriately
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct UsedTwice {
        #[relate(.shared)]
        first:  String,
        #[relate(.shared)]
        second: String,
    }

    #[test]
    fn test_auto_detects_multiple_usage() {
        let source = Source {
            shared: "shared value".to_string(),
        };

        // This works because auto mode clones when field is used multiple times
        let target: UsedTwice = source.into();

        assert_eq!(target.first, "shared value");
        assert_eq!(target.second, "shared value");
    }
}

// =============================================================================
// Clone Mode with Bidirectional
// =============================================================================

mod bidirectional_clone_modes {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Target, both, cloned)]
    struct Source {
        name:  String,
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        name:  String,
        value: i32,
    }

    #[test]
    fn test_bidirectional_with_cloned() {
        let source = Source {
            name:  "test".to_string(),
            value: 42,
        };

        let target: Target = source.clone().into();
        let back: Source = target.into();

        assert_eq!(back.name, "test");
        assert_eq!(back.value, 42);
    }
}

// =============================================================================
// Clone Mode Combined with All Options
// =============================================================================

mod combined_options {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Source {
        name:   String,
        count:  i32,
        active: bool,
    }

    // Combining: both + cloned
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, both, cloned)]
    struct BothCloned {
        name:   String,
        count:  i32,
        active: bool,
    }

    #[test]
    fn test_both_and_cloned_combined() {
        let source = Source {
            name:   "combined".to_string(),
            count:  10,
            active: true,
        };

        let target: BothCloned = source.clone().into();
        assert_eq!(target.name, "combined");

        let back: Source = target.into();
        assert_eq!(back.name, "combined");
    }
}

// =============================================================================
// Move Mode with Copy Types
// =============================================================================

mod move_with_copy_types {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        x: i32,
        y: i32,
    }

    // Move mode works well with Copy types - no cloning overhead
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, move)]
    struct Target {
        x: i32,
        y: i32,
    }

    #[test]
    fn test_move_with_copy_types() {
        let source = Source { x: 10, y: 20 };

        let target: Target = source.into();

        assert_eq!(target.x, 10);
        assert_eq!(target.y, 20);
    }

    #[test]
    fn test_move_from_ref_still_clones() {
        // From<&Source> always clones, regardless of move mode
        let source = Source { x: 30, y: 40 };

        let target: Target = (&source).into();

        assert_eq!(target.x, 30);
        assert_eq!(target.y, 40);
        // source still available
        assert_eq!(source.x, 30);
    }
}

// =============================================================================
// Edge Case: Empty Struct
// =============================================================================

mod empty_struct {
    use super::*;

    #[derive(Debug, Clone)]
    struct EmptySource {}

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(EmptySource, cloned)]
    struct EmptyTarget {}

    #[test]
    fn test_empty_struct_with_cloned() {
        let source = EmptySource {};
        let target: EmptyTarget = source.into();
        assert_eq!(target, EmptyTarget {});
    }
}
