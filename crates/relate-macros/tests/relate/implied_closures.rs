//! Tests for `with = expr` syntax and expression transforms.

use relate::relate_structs;

// Test `with = expr` with field access and expressions
mod field_expressions {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        value: i32,
        name:  String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        doubled:   i32,
        uppercase: String,
        wrapped:   Option<i32>,
    }

    relate_structs! {
        Source ~> Target {
            doubled: with = .value * 2;
            uppercase: with = .name.to_uppercase();
            wrapped: with = Some(.value);
        }
    }

    #[test]
    fn test_arithmetic_expression() {
        let source = Source {
            value: 21,
            name:  "test".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.doubled, 42);
    }

    #[test]
    fn test_method_expression() {
        let source = Source {
            value: 0,
            name:  "hello".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.uppercase, "HELLO");
    }

    #[test]
    fn test_constructor_expression() {
        let source = Source {
            value: 100,
            name:  "x".to_string(),
        };
        let target: Target = source.into();
        assert_eq!(target.wrapped, Some(100));
    }
}

// Test `with = expr` for cross-field access using `.field` syntax
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

    relate_structs! {
        Setting ~> SettingResponse {
            // Hide value if encrypted - use `with = expr` with `.field` for source access
            encrypted;
            value: with = if .encrypted { None } else { Some(.value.clone()) };
        }
    }

    #[test]
    fn test_unencrypted_visible() {
        let setting = Setting {
            value:     "https://api.example.com".to_string(),
            encrypted: false,
        };
        let response: SettingResponse = setting.into();
        assert_eq!(response.value, Some("https://api.example.com".to_string()));
        assert!(!response.encrypted);
    }

    #[test]
    fn test_encrypted_hidden() {
        let setting = Setting {
            value:     "secret123".to_string(),
            encrypted: true,
        };
        let response: SettingResponse = setting.into();
        assert_eq!(response.value, None);
        assert!(response.encrypted);
    }
}

// Test with constructors (None/Some)
mod option_constructors {
    use super::*;

    #[derive(Debug, Clone)]
    struct Input {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Output {
        some_value: Option<i32>,
        none_value: Option<String>,
    }

    relate_structs! {
        Input ~> Output {
            some_value: with = Some(.value * 2);
            none_value: default = None;
        }
    }

    #[test]
    fn test_some_constructor() {
        let input = Input { value: 21 };
        let output: Output = input.into();
        assert_eq!(output.some_value, Some(42));
        assert_eq!(output.none_value, None);
    }
}

// Test Result constructors
mod result_constructors {
    use super::*;

    #[derive(Debug, Clone)]
    struct Input {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Output {
        ok_value:  Result<i32, String>,
        err_value: Result<i32, String>,
    }

    relate_structs! {
        Input ~> Output {
            ok_value: with = Ok(.value);
            err_value: default = Err("error".to_string());
        }
    }

    #[test]
    fn test_result_constructors() {
        let input = Input { value: 42 };
        let output: Output = input.into();
        assert_eq!(output.ok_value, Ok(42));
        assert_eq!(output.err_value, Err("error".to_string()));
    }
}

// Test numeric expressions with `with = expr`
mod numeric_expressions {
    use super::*;

    #[derive(Debug, Clone)]
    struct Numbers {
        a: i32,
        b: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Computed {
        sum:     i32,
        product: i32,
        literal: i32,
    }

    relate_structs! {
        Numbers ~> Computed {
            sum: with = .a + 10;
            product: with = .b * 3;
            literal: default = 42;
        }
    }

    #[test]
    fn test_numeric_expressions() {
        let nums = Numbers { a: 5, b: 7 };
        let computed: Computed = nums.into();
        assert_eq!(computed.sum, 15);
        assert_eq!(computed.product, 21);
        assert_eq!(computed.literal, 42);
    }
}

// Test boolean expressions with `with = expr`
mod boolean_expressions {
    use super::*;

    #[derive(Debug, Clone)]
    struct Flags {
        enabled: bool,
        count:   i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Status {
        active:    bool,
        has_items: bool,
    }

    relate_structs! {
        Flags ~> Status {
            active: with = .enabled;
            has_items: with = .count > 0;
        }
    }

    #[test]
    fn test_boolean_expressions() {
        let flags = Flags {
            enabled: true,
            count:   5,
        };
        let status: Status = flags.into();
        assert!(status.active);
        assert!(status.has_items);
    }

    #[test]
    fn test_boolean_false() {
        let flags = Flags {
            enabled: false,
            count:   0,
        };
        let status: Status = flags.into();
        assert!(!status.active);
        assert!(!status.has_items);
    }
}
