//! Tests for the `_.field` source access syntax.

use relate::Relate;

// =============================================================================
// Basic Source Access Tests
// =============================================================================

mod basic_source_access {
    use super::*;

    #[derive(Debug, Clone)]
    struct Wrapper {
        data: Inner,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Inner {
        name:  String,
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Wrapper)]
    struct Flat {
        #[relate(.data.name)]
        name:  String,
        #[relate(.data.value)]
        value: i32,
    }

    #[test]
    fn test_nested_field_access() {
        let wrapper = Wrapper {
            data: Inner {
                name:  "nested".to_string(),
                value: 42,
            },
        };

        let flat: Flat = wrapper.into();

        assert_eq!(flat.name, "nested");
        assert_eq!(flat.value, 42);
    }

    #[test]
    fn test_nested_field_access_from_ref() {
        let wrapper = Wrapper {
            data: Inner {
                name:  "nested_ref".to_string(),
                value: 100,
            },
        };

        let flat: Flat = (&wrapper).into();

        assert_eq!(flat.name, "nested_ref");
        assert_eq!(flat.value, 100);
        // wrapper still available
        assert_eq!(wrapper.data.name, "nested_ref");
    }
}

// =============================================================================
// Deep Nesting Tests
// =============================================================================

mod deep_nesting {
    use super::*;

    #[derive(Debug, Clone)]
    struct Level1 {
        level2: Level2,
    }

    #[derive(Debug, Clone)]
    struct Level2 {
        level3: Level3,
    }

    #[derive(Debug, Clone)]
    struct Level3 {
        value: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Level1)]
    struct DeepFlat {
        #[relate(.level2.level3.value)]
        deeply_nested: String,
    }

    #[test]
    fn test_three_level_deep_access() {
        let source = Level1 {
            level2: Level2 {
                level3: Level3 {
                    value: "deep".to_string(),
                },
            },
        };

        let flat: DeepFlat = source.into();
        assert_eq!(flat.deeply_nested, "deep");
    }
}

// =============================================================================
// Method Calls on Source Access
// =============================================================================

mod method_calls {
    use super::*;

    #[derive(Debug, Clone)]
    struct Container {
        data: DataHolder,
    }

    #[derive(Debug, Clone)]
    struct DataHolder {
        items: Vec<String>,
        name:  String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Container)]
    struct Summary {
        #[relate(.data.items.len())]
        item_count:  usize,
        #[relate(.data.name.to_uppercase())]
        upper_name:  String,
        #[relate(.data.name.len())]
        name_length: usize,
    }

    #[test]
    fn test_method_calls_on_nested_fields() {
        let container = Container {
            data: DataHolder {
                items: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                name:  "hello".to_string(),
            },
        };

        let summary: Summary = container.into();

        assert_eq!(summary.item_count, 3);
        assert_eq!(summary.upper_name, "HELLO");
        assert_eq!(summary.name_length, 5);
    }
}

// =============================================================================
// Source Access with Clone Mode
// =============================================================================

mod with_clone_mode {
    use super::*;

    #[derive(Debug, Clone)]
    struct Outer {
        inner: Inner,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Inner {
        data: String,
    }

    // Test with struct-level cloned mode
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Outer, cloned)]
    struct WithClonedMode {
        #[relate(.inner.data)]
        data: String,
    }

    #[test]
    fn test_source_access_with_cloned_mode() {
        let outer = Outer {
            inner: Inner {
                data: "cloned".to_string(),
            },
        };

        let result: WithClonedMode = outer.into();
        assert_eq!(result.data, "cloned");
    }

    // Test with field-level move override
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Outer, cloned)]
    struct WithMoveOverride {
        #[relate(.inner.data, move)]
        data: String,
    }

    #[test]
    fn test_source_access_with_move_override() {
        let outer = Outer {
            inner: Inner {
                data: "moved".to_string(),
            },
        };

        let result: WithMoveOverride = outer.into();
        assert_eq!(result.data, "moved");
    }
}

// =============================================================================
// Optional Field Access
// =============================================================================

mod optional_fields {
    use super::*;

    #[derive(Debug, Clone)]
    struct MaybeData {
        value: Option<Inner>,
    }

    #[derive(Debug, Clone)]
    struct Inner {
        name: String,
    }

    // For complex expressions with closures, use `with = expr` syntax
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(MaybeData)]
    struct ExtractedOptional {
        #[relate(with = .value.as_ref().map(|v| v.name.clone()))]
        name: Option<String>,
    }

    #[test]
    fn test_optional_with_some() {
        let data = MaybeData {
            value: Some(Inner {
                name: "present".to_string(),
            }),
        };

        let result: ExtractedOptional = data.into();
        assert_eq!(result.name, Some("present".to_string()));
    }

    #[test]
    fn test_optional_with_none() {
        let data = MaybeData { value: None };

        let result: ExtractedOptional = data.into();
        assert_eq!(result.name, None);
    }
}

// =============================================================================
// Same Source Field Used Multiple Times
// =============================================================================

mod multiple_usage {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        data: SharedData,
    }

    #[derive(Debug, Clone)]
    struct SharedData {
        name: String,
    }

    // Clone detection works for ChainedAccess paths - same path is detected
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct MultiTarget {
        #[relate(.data.name)]
        name1:    String,
        #[relate(.data.name)]
        name2:    String,
        #[relate(.data.name.len())]
        name_len: usize,
    }

    #[test]
    fn test_same_nested_field_used_multiple_times() {
        let source = Source {
            data: SharedData {
                name: "shared".to_string(),
            },
        };

        let target: MultiTarget = source.into();

        assert_eq!(target.name1, "shared");
        assert_eq!(target.name2, "shared");
        assert_eq!(target.name_len, 6);
    }

    // Test that same source path used multiple times triggers clone detection
    #[derive(Debug, Clone)]
    struct SharedSource {
        value: Option<i32>,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(SharedSource)]
    struct SharedTarget {
        // Both access .value - should be detected as same source
        #[relate(.value.unwrap_or(0))]
        first:  i32,
        #[relate(.value.unwrap_or(1))]
        second: i32,
    }

    #[test]
    fn test_same_source_path_clones() {
        let source = SharedSource { value: Some(42) };
        let target: SharedTarget = source.into();
        assert_eq!(target.first, 42);
        assert_eq!(target.second, 42);

        let source_none = SharedSource { value: None };
        let target_none: SharedTarget = source_none.into();
        assert_eq!(target_none.first, 0);
        assert_eq!(target_none.second, 1);
    }
}

// =============================================================================
// Mixed Syntax: Regular Fields and Source Access
// =============================================================================

mod mixed_syntax {
    use super::*;

    #[derive(Debug, Clone)]
    struct MixedSource {
        top_level: String,
        nested:    NestedData,
    }

    #[derive(Debug, Clone)]
    struct NestedData {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(MixedSource)]
    struct MixedTarget {
        // Regular field mapping
        top_level:    String,
        // Source access for nested
        #[relate(.nested.value)]
        nested_value: i32,
    }

    #[test]
    fn test_mixed_regular_and_source_access() {
        let source = MixedSource {
            top_level: "top".to_string(),
            nested:    NestedData { value: 42 },
        };

        let target: MixedTarget = source.into();

        assert_eq!(target.top_level, "top");
        assert_eq!(target.nested_value, 42);
    }
}
