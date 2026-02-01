//! Tests for TryFrom generation with ~>? syntax.

use relate::{ConversionError, relate_structs};

// Test basic TryFrom with fallible method
mod basic_try_from {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        value: i32,
    }

    relate_structs! {
        Source ~>? Target {
            value: with = _.parse()?;
        }
    }

    #[test]
    fn test_try_from_success() {
        let source = Source {
            value: "42".to_string(),
        };
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 42);
    }

    #[test]
    fn test_try_from_failure() {
        let source = Source {
            value: "not a number".to_string(),
        };
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_ref() {
        let source = Source {
            value: "100".to_string(),
        };
        let result: Result<Target, ConversionError> = (&source).try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 100);
    }
}

// Test TryFrom with custom error type
mod custom_error {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        value: i32,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct MyError(String);

    impl From<std::num::ParseIntError> for MyError {
        fn from(e: std::num::ParseIntError) -> Self { MyError(e.to_string()) }
    }

    relate_structs! {
        Source ~>?[MyError] Target {
            value: with = _.parse()?;
        }
    }

    #[test]
    fn test_custom_error_success() {
        let source = Source {
            value: "42".to_string(),
        };
        let result: Result<Target, MyError> = source.try_into();
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_error_failure() {
        let source = Source {
            value: "invalid".to_string(),
        };
        let result: Result<Target, MyError> = source.try_into();
        assert!(result.is_err());
    }
}

// Test TryFrom with multiple fallible fields
mod multiple_fallible {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        port:  String,
        count: String,
        name:  String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        port:  u16,
        count: i32,
        name:  String,
    }

    relate_structs! {
        Source ~>? Target {
            port: with = _.parse()?;
            count: with = _.parse()?;
            name;  // Infallible
        }
    }

    #[test]
    fn test_multiple_fallible_success() {
        let source = Source {
            port:  "8080".to_string(),
            count: "42".to_string(),
            name:  "test".to_string(),
        };
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_ok());
        let target = result.unwrap();
        assert_eq!(target.port, 8080);
        assert_eq!(target.count, 42);
        assert_eq!(target.name, "test");
    }

    #[test]
    fn test_first_field_fails() {
        let source = Source {
            port:  "invalid".to_string(),
            count: "42".to_string(),
            name:  "test".to_string(),
        };
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_second_field_fails() {
        let source = Source {
            port:  "8080".to_string(),
            count: "invalid".to_string(),
            name:  "test".to_string(),
        };
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_err());
    }
}

// Test TryFrom with field access + transform
mod field_access_with_transform {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        raw_value: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        parsed: i32,
    }

    relate_structs! {
        Source ~>? Target {
            parsed: with = .raw_value.parse()?;
        }
    }

    #[test]
    fn test_field_access_with_fallible_transform() {
        let source = Source {
            raw_value: "123".to_string(),
        };
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().parsed, 123);
    }
}

// Test TryFrom with mix of fallible and defaults
mod fallible_with_defaults {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        value: i32,
        extra: String,
    }

    relate_structs! {
        Source ~>? Target {
            value: with = _.parse()?;
            extra: default = "default".to_string();
        }
    }

    #[test]
    fn test_fallible_with_default() {
        let source = Source {
            value: "42".to_string(),
        };
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_ok());
        let target = result.unwrap();
        assert_eq!(target.value, 42);
        assert_eq!(target.extra, "default");
    }
}

// Test regular From still works (no regression)
mod regular_from_still_works {
    use super::*;

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
            value;
        }
    }

    #[test]
    fn test_regular_from() {
        let source = Source { value: 42 };
        let target: Target = source.into();
        assert_eq!(target.value, 42);
    }
}

// Test auto TryFrom detection - using ~> with fallible transform auto-upgrades
// to TryFrom
mod auto_try_from_detection {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: String,
        count: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        value: i32,
        count: i32,
    }

    // Using `~>` instead of `~>?` - should auto-detect fallibility
    relate_structs! {
        Source ~> Target {
            value: with = _.parse()?;
            count: with = _.parse()?;
        }
    }

    #[test]
    fn test_auto_try_from_success() {
        let source = Source {
            value: "42".to_string(),
            count: "100".to_string(),
        };
        // Uses try_into! Not into!
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_ok());
        let target = result.unwrap();
        assert_eq!(target.value, 42);
        assert_eq!(target.count, 100);
    }

    #[test]
    fn test_auto_try_from_failure() {
        let source = Source {
            value: "not a number".to_string(),
            count: "100".to_string(),
        };
        let result: Result<Target, ConversionError> = source.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_try_from_ref() {
        let source = Source {
            value: "42".to_string(),
            count: "100".to_string(),
        };
        let result: Result<Target, ConversionError> = (&source).try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 42);
    }
}
