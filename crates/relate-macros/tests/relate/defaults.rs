//! Tests for default values in relate_structs!

use relate::relate_structs;

mod basic_defaults {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        name:     String,
        id:       Option<i32>,
        active:   bool,
        metadata: String,
    }

    relate_structs! {
        Source ~> Target {
            name;
            id: default = None;
            active: default = false;
            metadata: default = "none".to_string();
        }
    }

    #[test]
    fn test_defaults() {
        let source = Source {
            name: "test".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.name, "test");
        assert_eq!(target.id, None);
        assert!(!target.active);
        assert_eq!(target.metadata, "none");
    }
}

mod default_no_expr {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq, Default)]
    struct Target {
        value: i32,
        extra: String,
        count: i32,
    }

    // `default` without expression uses Default::default()
    relate_structs! {
        Source ~> Target {
            value;
            extra: default;
            count: default;
        }
    }

    #[test]
    fn test_default_no_expr() {
        let source = Source { value: 42 };
        let target: Target = source.into();
        assert_eq!(target.value, 42);
        assert_eq!(target.extra, "");
        assert_eq!(target.count, 0);
    }
}

mod multiple_fields {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        id:   i32,
        name: String,
        data: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        id:   i32,
        name: String,
        data: String,
    }

    // List fields individually (replaces old @pick syntax)
    relate_structs! {
        Source ~> Target {
            id;
            name;
            data;
        }
    }

    #[test]
    fn test_multiple_fields() {
        let source = Source {
            id:   1,
            name: "test".to_string(),
            data: "hello".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.id, 1);
        assert_eq!(target.name, "test");
        assert_eq!(target.data, "hello");
    }
}

mod fields_with_transform {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        id:     i32,
        name:   String,
        value:  Option<i32>,
        extras: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        id:     i32,
        name:   String,
        value:  i32,
        extras: String,
        added:  bool,
    }

    // Mix of identity, transforms, and defaults
    relate_structs! {
        Source ~> Target {
            id;
            name;
            extras;
            value: with = _.unwrap_or(0);
            added: default = true;
        }
    }

    #[test]
    fn test_fields_with_transform() {
        let source = Source {
            id:     42,
            name:   "test".to_string(),
            value:  Some(100),
            extras: "data".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.id, 42);
        assert_eq!(target.name, "test");
        assert_eq!(target.value, 100);
        assert_eq!(target.extras, "data");
        assert!(target.added);
    }
}
