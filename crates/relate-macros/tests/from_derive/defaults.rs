//! Tests for default values in Relate derive.

use relate::Relate;

#[derive(Debug, Clone)]
struct Source {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(Source)]
struct Target {
    name:        String,
    #[relate(default)]
    extra:       i32,
    #[relate(default = None)]
    optional:    Option<String>,
    #[relate(default = "default".to_string())]
    with_value:  String,
    #[relate(skip)]
    skipped:     i32,
    #[relate(default = false)]
    should_sync: bool,
}

// Test: same default expression used multiple times (hoisted to let binding)
mod duplicate_defaults {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    // Counter to track how many times the expression is evaluated
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn get_timestamp() -> i64 {
        COUNTER.fetch_add(1, Ordering::SeqCst);
        42 // Return a fixed value for testing
    }

    #[derive(Debug, Clone)]
    struct SimpleSource {
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(SimpleSource)]
    struct WithDuplicateDefaults {
        name:       String,
        #[relate(default = get_timestamp())]
        created_at: i64,
        #[relate(default = get_timestamp())]
        updated_at: i64,
    }

    #[test]
    fn test_duplicate_defaults_called_once() {
        COUNTER.store(0, Ordering::SeqCst);

        let source = SimpleSource {
            name: "test".to_string(),
        };

        let target: WithDuplicateDefaults = source.into();

        // Both fields should have the same value
        assert_eq!(target.created_at, target.updated_at);
        // The function should have been called only once (hoisted to let binding)!
        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_duplicate_defaults_from_ref() {
        COUNTER.store(0, Ordering::SeqCst);

        let source = SimpleSource {
            name: "test".to_string(),
        };

        let target: WithDuplicateDefaults = (&source).into();

        assert_eq!(target.created_at, target.updated_at);
        // The function should have been called only once (hoisted to let binding)!
        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn test_default_values() {
    let source = Source {
        name: "test".to_string(),
    };

    let target: Target = source.into();

    assert_eq!(target.name, "test");
    assert_eq!(target.extra, 0); // Default::default()
    assert_eq!(target.optional, None);
    assert_eq!(target.with_value, "default");
    assert_eq!(target.skipped, 0); // Default::default()
    assert!(!target.should_sync);
}

#[test]
fn test_from_ref_with_defaults() {
    let source = Source {
        name: "ref_test".to_string(),
    };

    let target: Target = (&source).into();

    assert_eq!(target.name, "ref_test");
    assert_eq!(target.extra, 0);
    assert_eq!(target.optional, None);

    // Source still usable
    assert_eq!(source.name, "ref_test");
}
