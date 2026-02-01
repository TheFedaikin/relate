//! Tests for transforms in Relate derive.

use relate::Relate;

// Test method-call transform
#[derive(Debug, Clone)]
struct OptionalSource {
    id:   Option<i32>,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(OptionalSource)]
struct RequiredTarget {
    #[relate(_.unwrap_or(0))]
    id:   i32,
    name: String,
}

#[test]
fn test_method_call_transform() {
    let source = OptionalSource {
        id:   Some(42),
        name: "test".to_string(),
    };

    let target: RequiredTarget = source.into();

    assert_eq!(target.id, 42);
    assert_eq!(target.name, "test");
}

#[test]
fn test_method_call_with_none() {
    let source = OptionalSource {
        id:   None,
        name: "none".to_string(),
    };

    let target: RequiredTarget = source.into();

    assert_eq!(target.id, 0);
}

// Test chained method calls
#[derive(Debug, Clone)]
struct RawText {
    content: String,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(RawText)]
struct ProcessedText {
    #[relate(_.trim().to_uppercase())]
    content: String,
}

#[test]
fn test_chained_method_calls() {
    let raw = RawText {
        content: "  hello world  ".to_string(),
    };

    let processed: ProcessedText = raw.into();

    assert_eq!(processed.content, "HELLO WORLD");
}

// Test `with = expr` transform for arithmetic
#[derive(Debug, Clone)]
struct Numbers {
    value: i32,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(Numbers)]
struct Doubled {
    #[relate(with = _ * 2)]
    value: i32,
}

#[test]
fn test_with_expr_arithmetic() {
    let numbers = Numbers { value: 21 };

    let doubled: Doubled = numbers.into();

    assert_eq!(doubled.value, 42);
}

// Test function call via with = expr
mod transforms {
    pub fn double(x: i32) -> i32 { x * 2 }
}

#[derive(Debug, Clone)]
struct Input {
    value: i32,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(Input)]
struct Output {
    #[relate(with = transforms::double(_))]
    value: i32,
}

#[test]
fn test_function_call_with_expr() {
    let input = Input { value: 21 };

    let output: Output = input.into();

    assert_eq!(output.value, 42);
}

// Test nested field access with dot syntax (not method call)
mod nested_field_access {
    use relate::Relate;

    struct Wrapper {
        name: Inner,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Inner {
        data:  String,
        value: i32,
    }

    #[derive(Debug, PartialEq, Relate)]
    #[relate(Wrapper)]
    struct Flat {
        #[relate(.name.data)]
        name:  String,
        #[relate(.name.value)]
        value: i32,
    }

    #[test]
    fn test_dot_field_access() {
        let wrapper = Wrapper {
            name: Inner {
                data:  "hello".to_string(),
                value: 42,
            },
        };

        let flat: Flat = wrapper.into();
        assert_eq!(flat.name, "hello");
        assert_eq!(flat.value, 42);
    }
}
