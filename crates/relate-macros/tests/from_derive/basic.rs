//! Basic tests for the Relate derive macro.

use relate::Relate;

// Source struct - no derive needed
#[derive(Debug, Clone)]
struct Source {
    name:  String,
    value: i32,
}

// Target struct - derive Relate
#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(Source)]
struct Target {
    name:  String,
    value: i32,
}

// Test: same source field used multiple times (should auto-clone)
mod duplicate_field_test {
    use super::*;

    #[derive(Debug, Clone)]
    struct Store {
        id:   String,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Store)]
    struct Warehouse {
        #[relate(.id)]
        moysklad_id: String,
        name:        String,
        #[relate(.name)] // Same field as `name` - should auto-clone
        sync_name: String,
    }

    #[test]
    fn test_same_source_field_used_twice_owned() {
        let store = Store {
            id:   "abc123".to_string(),
            name: "Main Warehouse".to_string(),
        };

        // This should work even though `name` is used for both `name` and `sync_name`
        let warehouse: Warehouse = store.into();

        assert_eq!(warehouse.moysklad_id, "abc123");
        assert_eq!(warehouse.name, "Main Warehouse");
        assert_eq!(warehouse.sync_name, "Main Warehouse");
    }

    #[test]
    fn test_same_source_field_used_twice_ref() {
        let store = Store {
            id:   "xyz789".to_string(),
            name: "Secondary".to_string(),
        };

        let warehouse: Warehouse = (&store).into();

        assert_eq!(warehouse.moysklad_id, "xyz789");
        assert_eq!(warehouse.name, "Secondary");
        assert_eq!(warehouse.sync_name, "Secondary");

        // Store still available
        assert_eq!(store.name, "Secondary");
    }
}

#[test]
fn test_basic_auto_map() {
    let source = Source {
        name:  "test".to_string(),
        value: 42,
    };

    let target: Target = source.into();

    assert_eq!(target.name, "test");
    assert_eq!(target.value, 42);
}

#[test]
fn test_from_ref() {
    let source = Source {
        name:  "test".to_string(),
        value: 42,
    };

    // Convert from reference - source is NOT consumed
    let target: Target = (&source).into();

    assert_eq!(target.name, "test");
    assert_eq!(target.value, 42);

    // source is still usable!
    assert_eq!(source.name, "test");
}

#[test]
fn test_from_ref_multiple_uses() {
    let source = Source {
        name:  "multi".to_string(),
        value: 100,
    };

    // Can convert multiple times from the same source
    let t1: Target = (&source).into();
    let t2: Target = (&source).into();
    let t3: Target = source.into(); // Finally consume it

    assert_eq!(t1, t2);
    assert_eq!(t2, t3);
}
