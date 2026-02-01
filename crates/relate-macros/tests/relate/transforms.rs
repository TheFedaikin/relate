//! Tests for field transforms in relate_structs!

use relate::relate_structs;

// Test method call transforms on same-named field
mod method_call {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        id:   Option<i32>,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        id:   i32,
        name: String,
    }

    relate_structs! {
        Source ~> Target {
            id: with = _.unwrap_or(0);
            name;
        }
    }

    #[test]
    fn test_method_transform() {
        let source = Source {
            id:   Some(42),
            name: "test".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.id, 42);
        assert_eq!(target.name, "test");
    }

    #[test]
    fn test_method_transform_default() {
        let source = Source {
            id:   None,
            name: "test".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.id, 0);
    }
}

// Test transforms with arithmetic expression
mod expr_transform {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        count: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        doubled: i32,
    }

    // Using `with = expr` syntax to access source field and compute target
    // `.count` accesses source.count, result goes to doubled
    relate_structs! {
        Source ~> Target {
            doubled: with = .count * 2;
        }
    }

    #[test]
    fn test_expr_transform() {
        let source = Source { count: 21 };
        let target: Target = source.into();
        assert_eq!(target.doubled, 42);
    }
}

// Test cross-field conditional logic with `.field` syntax
mod cross_field_conditional {
    use super::*;

    #[derive(Debug, Clone)]
    struct Setting {
        value:     String,
        encrypted: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct SettingResponse {
        value:     Option<String>,
        encrypted: bool,
    }

    // Using `.field` to access other source fields
    relate_structs! {
        Setting ~> SettingResponse {
            value: with = if .encrypted { None } else { Some(.value.clone()) };
            encrypted;
        }
    }

    #[test]
    fn test_encrypted_hidden() {
        let setting = Setting {
            value:     "secret".to_string(),
            encrypted: true,
        };
        let response: SettingResponse = setting.into();
        assert_eq!(response.value, None);
        assert!(response.encrypted);
    }

    #[test]
    fn test_unencrypted_shown() {
        let setting = Setting {
            value:     "public".to_string(),
            encrypted: false,
        };
        let response: SettingResponse = setting.into();
        assert_eq!(response.value, Some("public".to_string()));
        assert!(!response.encrypted);
    }
}

// Test wrapping in constructor using with = Some(_)
mod wrap_in_constructor {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        value: Option<i32>,
    }

    // Use `with = Some(_)` to wrap the field value
    relate_structs! {
        Source ~> Target {
            value: with = Some(_);
        }
    }

    #[test]
    fn test_wrap_in_some() {
        let source = Source { value: 42 };
        let target: Target = source.into();
        assert_eq!(target.value, Some(42));
    }

    #[test]
    fn test_wrap_in_some_from_ref() {
        let source = Source { value: 42 };
        let target: Target = (&source).into();
        assert_eq!(target.value, Some(42));
    }
}

// Test qualified function path (multiple segments)
mod qualified_function_path {
    use super::*;

    mod transforms {
        pub fn double(x: i32) -> i32 { x * 2 }
    }

    #[derive(Debug, Clone)]
    struct Source {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        value: i32,
    }

    relate_structs! {
        Source ~> Target {
            value: with = transforms::double(_);
        }
    }

    #[test]
    fn test_qualified_function_path() {
        let source = Source { value: 21 };
        let target: Target = source.into();
        assert_eq!(target.value, 42);
    }
}

// Test From<&T> explicitly
mod from_ref {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        id:   i32,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        id:   i32,
        name: String,
    }

    relate_structs! {
        Source ~> Target {
            id;
            name;
        }
    }

    #[test]
    fn test_from_ref_preserves_source() {
        let source = Source {
            id:   42,
            name: "test".to_string(),
        };

        // Convert from reference - source should still be usable
        let target: Target = (&source).into();
        assert_eq!(target.id, 42);
        assert_eq!(target.name, "test");

        // Source still accessible
        assert_eq!(source.id, 42);
        assert_eq!(source.name, "test");
    }

    #[test]
    fn test_from_owned() {
        let source = Source {
            id:   42,
            name: "test".to_string(),
        };

        let target: Target = source.into();
        assert_eq!(target.id, 42);
        assert_eq!(target.name, "test");
        // source is now consumed
    }
}

// Test `with = expr` for cross-field arithmetic
mod with_expr {
    use super::*;

    #[derive(Debug, Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Distance {
        sum:      i32,
        product:  i32,
        combined: String,
    }

    // Using `with = expr` syntax with `.field` for source access
    relate_structs! {
        Point ~> Distance {
            sum: with = .x + .y;
            product: with = .x * .y;
            combined: with = format!("({}, {})", .x, .y);
        }
    }

    #[test]
    fn test_with_expr_simple() {
        let point = Point { x: 3, y: 4 };
        let dist: Distance = point.into();
        assert_eq!(dist.sum, 7);
        assert_eq!(dist.product, 12);
        assert_eq!(dist.combined, "(3, 4)");
    }

    #[test]
    fn test_with_expr_from_ref() {
        let point = Point { x: 3, y: 4 };
        let dist: Distance = (&point).into();
        assert_eq!(dist.sum, 7);
        assert_eq!(dist.product, 12);
        // point is still usable
        assert_eq!(point.x, 3);
    }
}

// Test chained access syntax: `.path.field`
mod chained_access {
    use super::*;

    #[derive(Debug, Clone)]
    struct Inner {
        value: i32,
        name:  String,
    }

    #[derive(Debug, Clone)]
    struct Source {
        data:   Inner,
        active: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        value:  i32,
        name:   String,
        active: bool,
    }

    relate_structs! {
        Source ~> Target {
            value: with = .data.value;
            name: with = .data.name.clone();
            active;
        }
    }

    #[test]
    fn test_chained_access() {
        let source = Source {
            data:   Inner {
                value: 42,
                name:  "test".to_string(),
            },
            active: true,
        };
        let target: Target = source.into();
        assert_eq!(target.value, 42);
        assert_eq!(target.name, "test");
        assert!(target.active);
    }

    #[test]
    fn test_chained_access_from_ref() {
        let source = Source {
            data:   Inner {
                value: 100,
                name:  "hello".to_string(),
            },
            active: false,
        };
        let target: Target = (&source).into();
        assert_eq!(target.value, 100);
        assert_eq!(target.name, "hello");
        assert!(!target.active);
        // source still usable
        assert_eq!(source.data.value, 100);
    }
}

// Test chained access with method calls
mod chained_access_with_method {
    use super::*;

    #[derive(Debug, Clone)]
    struct Inner {
        text: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct Source {
        data: Inner,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        text: String,
    }

    relate_structs! {
        Source ~> Target {
            text: with = .data.text.clone().unwrap_or_default();
        }
    }

    #[test]
    fn test_chained_method_call() {
        let source = Source {
            data: Inner {
                text: Some("hello".to_string()),
            },
        };
        let target: Target = source.into();
        assert_eq!(target.text, "hello");
    }

    #[test]
    fn test_chained_method_default() {
        let source = Source {
            data: Inner { text: None },
        };
        let target: Target = source.into();
        assert_eq!(target.text, "");
    }
}
