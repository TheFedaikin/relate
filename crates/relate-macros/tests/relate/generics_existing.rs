//! Tests for generic structs with relate_structs!

use relate::relate_structs;

// Test: existing generic structs WITH Clone bound
mod existing_generic_with_bound {
    use super::*;

    #[derive(Debug, Clone)]
    struct Container<T: Clone> {
        value: T,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Wrapper<T: Clone> {
        inner: T,
    }

    // Clone bound required for From<&T> impl
    relate_structs! {
        Container<T: Clone> ~> Wrapper<T: Clone> {
            inner: with = .value.clone();
        }
    }

    #[test]
    fn test_generic_i32() {
        let c: Container<i32> = Container { value: 42 };
        let w: Wrapper<i32> = c.into();
        assert_eq!(w.inner, 42);
    }

    #[test]
    fn test_generic_string() {
        let c: Container<String> = Container {
            value: "hello".to_string(),
        };
        let w: Wrapper<String> = c.into();
        assert_eq!(w.inner, "hello");
    }

    #[test]
    fn test_from_ref() {
        let c: Container<i32> = Container { value: 100 };
        let w: Wrapper<i32> = (&c).into();
        assert_eq!(w.inner, 100);
    }
}
