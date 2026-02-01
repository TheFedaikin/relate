//! Tests for field renames in relate_structs!

use relate::relate_structs;

// Test simple rename using `with = .source_field` syntax
mod simple {
    use super::*;

    #[derive(Debug, Clone)]
    struct ApiStore {
        id:   String,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Warehouse {
        moysklad_id: String,
        name:        String,
    }

    // Syntax: target_field: with = .source_field;
    relate_structs! {
        ApiStore ~> Warehouse {
            moysklad_id: with = .id.clone();
            name;
        }
    }

    #[test]
    fn test_rename() {
        let store = ApiStore {
            id:   "abc123".to_string(),
            name: "Main".to_string(),
        };
        let warehouse: Warehouse = store.into();
        assert_eq!(warehouse.moysklad_id, "abc123");
        assert_eq!(warehouse.name, "Main");
    }
}
