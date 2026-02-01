//! Tests for automatic TryFrom inference and explicit error types.

use relate::Relate;

// =============================================================================
// Auto TryFrom Detection via `?` Operator
// =============================================================================

mod auto_detection {
    use super::*;

    #[derive(Debug, Clone)]
    struct RawConfig {
        port: String,
        host: String,
    }

    // The `?` in `.parse()?` should trigger TryFrom generation
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(RawConfig)]
    struct Config {
        #[relate(_.parse()?)]
        port: u16,
        host: String,
    }

    #[test]
    fn test_auto_try_from_success() {
        let raw = RawConfig {
            port: "8080".to_string(),
            host: "localhost".to_string(),
        };

        let config: Result<Config, _> = raw.try_into();
        let config = config.expect("should parse successfully");

        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "localhost");
    }

    #[test]
    fn test_auto_try_from_failure() {
        let raw = RawConfig {
            port: "not_a_number".to_string(),
            host: "localhost".to_string(),
        };

        let result: Result<Config, _> = raw.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_try_from_ref() {
        let raw = RawConfig {
            port: "3000".to_string(),
            host: "127.0.0.1".to_string(),
        };

        let config: Result<Config, _> = (&raw).try_into();
        let config = config.expect("should parse successfully");

        assert_eq!(config.port, 3000);
        // raw still available
        assert_eq!(raw.host, "127.0.0.1");
    }
}

// =============================================================================
// Explicit Error Type
// =============================================================================

mod explicit_error {
    use super::*;

    #[derive(Debug, Clone)]
    struct Input {
        value: String,
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct MyError(String);

    impl From<std::num::ParseIntError> for MyError {
        fn from(e: std::num::ParseIntError) -> Self { MyError(e.to_string()) }
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Input, error = MyError)]
    struct Output {
        #[relate(_.parse()?)]
        value: i32,
    }

    #[test]
    fn test_explicit_error_type() {
        let input = Input {
            value: "invalid".to_string(),
        };

        let result: Result<Output, MyError> = input.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.0.contains("invalid"));
    }
}

// =============================================================================
// Multiple Fallible Fields
// =============================================================================

mod multiple_fallible {
    use super::*;

    #[derive(Debug, Clone)]
    struct RawData {
        x: String,
        y: String,
        z: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(RawData)]
    struct Parsed {
        #[relate(_.parse()?)]
        x: i32,
        #[relate(_.parse()?)]
        y: i32,
        #[relate(_.parse()?)]
        z: i32,
    }

    #[test]
    fn test_multiple_fallible_all_succeed() {
        let raw = RawData {
            x: "1".to_string(),
            y: "2".to_string(),
            z: "3".to_string(),
        };

        let parsed: Result<Parsed, _> = raw.try_into();
        let parsed = parsed.expect("all should parse");

        assert_eq!(parsed.x, 1);
        assert_eq!(parsed.y, 2);
        assert_eq!(parsed.z, 3);
    }

    #[test]
    fn test_multiple_fallible_first_fails() {
        let raw = RawData {
            x: "bad".to_string(),
            y: "2".to_string(),
            z: "3".to_string(),
        };

        let result: Result<Parsed, _> = raw.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_fallible_middle_fails() {
        let raw = RawData {
            x: "1".to_string(),
            y: "bad".to_string(),
            z: "3".to_string(),
        };

        let result: Result<Parsed, _> = raw.try_into();
        assert!(result.is_err());
    }
}

// =============================================================================
// Mixed Fallible and Infallible Fields
// =============================================================================

mod mixed_fields {
    use super::*;

    #[derive(Debug, Clone)]
    struct MixedSource {
        number_str:  String,
        regular_str: String,
        copy_val:    i32,
        another_str: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(MixedSource)]
    struct MixedTarget {
        #[relate(_.parse()?)]
        number_str:  i32,
        regular_str: String,
        copy_val:    i32,
        #[relate(_.to_uppercase())]
        another_str: String,
    }

    #[test]
    fn test_mixed_fallible_infallible() {
        let source = MixedSource {
            number_str:  "42".to_string(),
            regular_str: "hello".to_string(),
            copy_val:    100,
            another_str: "world".to_string(),
        };

        let target: Result<MixedTarget, _> = source.try_into();
        let target = target.expect("should succeed");

        assert_eq!(target.number_str, 42);
        assert_eq!(target.regular_str, "hello");
        assert_eq!(target.copy_val, 100);
        assert_eq!(target.another_str, "WORLD");
    }
}

// =============================================================================
// TryFrom with Clone Modes
// =============================================================================

mod with_clone_modes {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct TargetCloned {
        #[relate(_.parse()?)]
        value: i32,
    }

    #[test]
    fn test_try_from_with_cloned_mode() {
        let source = Source {
            value: "123".to_string(),
        };

        let target: Result<TargetCloned, _> = source.try_into();
        let target = target.expect("should parse");

        assert_eq!(target.value, 123);
    }
}

// =============================================================================
// Nested Fallible Access
// =============================================================================

mod nested_fallible {
    use super::*;

    #[derive(Debug, Clone)]
    struct Wrapper {
        data: Data,
    }

    #[derive(Debug, Clone)]
    struct Data {
        num_str: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Wrapper)]
    struct Flat {
        #[relate(.data.num_str.parse::<i32>()?)]
        num: i32,
    }

    #[test]
    fn test_nested_source_access_fallible() {
        let wrapper = Wrapper {
            data: Data {
                num_str: "999".to_string(),
            },
        };

        let flat: Result<Flat, _> = wrapper.try_into();
        let flat = flat.expect("should parse");

        assert_eq!(flat.num, 999);
    }
}

// =============================================================================
// Fallible with Default Fields
// =============================================================================

mod fallible_with_defaults {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct WithDefault {
        #[relate(_.parse()?)]
        value: i32,
        #[relate(default)]
        extra: i32,
    }

    #[test]
    fn test_fallible_with_default_field() {
        let source = Source {
            value: "50".to_string(),
        };

        let target: Result<WithDefault, _> = source.try_into();
        let target = target.expect("should succeed");

        assert_eq!(target.value, 50);
        assert_eq!(target.extra, 0);
    }
}

// =============================================================================
// Option::ok_or Pattern
// =============================================================================

mod option_ok_or {
    use super::*;

    #[derive(Debug, Clone)]
    struct MaybeValue {
        opt: Option<String>,
    }

    // Error type that can be created from static str
    #[derive(Debug, Clone)]
    struct RequiredError;

    impl From<&str> for RequiredError {
        fn from(_: &str) -> Self { RequiredError }
    }

    // Using `._.clone()` pattern where `_` expands to `opt` (the field name)
    // This generates: src.opt.clone().ok_or("missing value")?
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(MaybeValue, error = RequiredError)]
    struct Required {
        #[relate(._.clone().ok_or("missing value")?)]
        opt: String,
    }

    #[test]
    fn test_option_ok_or_some() {
        let source = MaybeValue {
            opt: Some("present".to_string()),
        };

        let target: Result<Required, _> = source.try_into();
        let target = target.expect("should have value");

        assert_eq!(target.opt, "present");
    }

    #[test]
    fn test_option_ok_or_none() {
        let source = MaybeValue { opt: None };

        let result: Result<Required, RequiredError> = source.try_into();
        assert!(result.is_err());
    }
}

// =============================================================================
// Explicit try_from Keyword
// =============================================================================

mod explicit_try_from_keyword {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        name: String,
        age:  i32,
    }

    // Using explicit `try_from` keyword without any fallible fields
    // Should generate TryFrom even though no `?` operator is present
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, try_from)]
    struct Target {
        name: String,
        age:  i32,
    }

    #[test]
    fn test_explicit_try_from_keyword() {
        let source = Source {
            name: "Alice".to_string(),
            age:  30,
        };

        // This should compile and use TryFrom, not From
        let target: Result<Target, _> = source.try_into();
        let target = target.expect("should succeed");

        assert_eq!(target.name, "Alice");
        assert_eq!(target.age, 30);
    }

    #[test]
    fn test_explicit_try_from_ref() {
        let source = Source {
            name: "Bob".to_string(),
            age:  25,
        };

        // Reference conversion
        let target: Result<Target, _> = (&source).try_into();
        let target = target.expect("should succeed");

        assert_eq!(target.name, "Bob");
        assert_eq!(target.age, 25);
    }
}

// =============================================================================
// Explicit try_from with Custom Error
// =============================================================================

mod explicit_try_from_with_error {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: String,
    }

    #[derive(Debug, Clone)]
    struct CustomError;

    // Using try_from with custom error type
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, try_from = CustomError)]
    struct Target {
        value: String,
    }

    #[test]
    fn test_try_from_with_error_type() {
        let source = Source {
            value: "hello".to_string(),
        };

        // Should use TryFrom with CustomError as error type
        let result: Result<Target, CustomError> = source.try_into();
        assert!(result.is_ok());
    }
}

// =============================================================================
// Closure with Fallible Result
// =============================================================================

mod closure_fallible {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        data: String,
    }

    // Using with = expr that returns Result
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct Target {
        #[relate(with = .data.parse::<i32>()?)]
        data: i32,
    }

    #[test]
    fn test_with_expr_fallible() {
        let source = Source {
            data: "777".to_string(),
        };

        let target: Result<Target, _> = source.try_into();
        let target = target.expect("should parse");

        assert_eq!(target.data, 777);
    }
}
