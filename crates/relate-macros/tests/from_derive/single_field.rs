//! Tests for single-field structs.

use relate::Relate;

/// Test single-field struct conversion
mod basic {
    use super::*;

    #[derive(Debug, Clone)]
    struct SingleSource {
        value: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(SingleSource)]
    struct SingleTarget {
        value: String,
    }

    #[test]
    fn test_single_field_owned() {
        let source = SingleSource {
            value: "test".to_string(),
        };
        let target: SingleTarget = source.into();
        assert_eq!(target.value, "test");
    }

    #[test]
    fn test_single_field_ref() {
        let source = SingleSource {
            value: "test".to_string(),
        };
        let target: SingleTarget = (&source).into();
        assert_eq!(target.value, "test");
        // Original still usable
        assert_eq!(source.value, "test");
    }
}

/// Test single-field with transform
mod single_field_transform {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct Target {
        #[relate(_.to_uppercase())]
        name: String,
    }

    #[test]
    fn test_single_field_with_transform() {
        let source = Source {
            name: "hello".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.name, "HELLO");
    }
}

/// Test single-field with default (no fields from source)
mod single_field_default {
    use super::*;

    #[derive(Debug, Clone)]
    struct Empty {}

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Empty)]
    struct WithDefault {
        #[relate(default)]
        value: i32,
    }

    #[test]
    fn test_single_field_default_only() {
        let source = Empty {};
        let target: WithDefault = source.into();
        assert_eq!(target.value, 0);
    }
}

/// Test single Copy field
mod single_copy_field {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct CopySource {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(CopySource)]
    struct CopyTarget {
        #[relate(copy)]
        value: i32,
    }

    #[test]
    fn test_single_copy_field() {
        let source = CopySource { value: 42 };
        let target: CopyTarget = source.into();
        assert_eq!(target.value, 42);
    }

    #[test]
    fn test_single_copy_field_ref() {
        let source = CopySource { value: 42 };
        let target: CopyTarget = (&source).into();
        assert_eq!(target.value, 42);
    }
}
