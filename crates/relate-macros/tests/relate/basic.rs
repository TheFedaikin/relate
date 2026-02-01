//! Basic tests for the unified relate_structs! macro.

use relate::relate_structs;

// Test Pattern 1: Existing structs with explicit fields
mod existing_structs {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Source {
        name:  String,
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Target {
        name:  String,
        value: i32,
    }

    relate_structs! {
        Source ~> Target {
            name;
            value;
        }
    }

    #[test]
    fn test_forward_conversion() {
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
            name:  "ref_test".to_string(),
            value: 100,
        };
        let target: Target = (&source).into();
        assert_eq!(target.name, "ref_test");
        assert_eq!(source.name, "ref_test"); // Source still usable
    }
}

// Test Pattern 2: Bidirectional with ~
mod bidirectional {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Barcodes {
        ean13: Option<String>,
        ean8:  Option<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct DbBarcodes {
        ean13: Option<String>,
        ean8:  Option<String>,
    }

    relate_structs! {
        Barcodes ~ DbBarcodes {
            ean13;
            ean8;
        }
    }

    #[test]
    fn test_forward() {
        let b = Barcodes {
            ean13: Some("123".to_string()),
            ean8:  None,
        };
        let db: DbBarcodes = b.into();
        assert_eq!(db.ean13, Some("123".to_string()));
    }

    #[test]
    fn test_backward() {
        let db = DbBarcodes {
            ean13: None,
            ean8:  Some("456".to_string()),
        };
        let b: Barcodes = db.into();
        assert_eq!(b.ean8, Some("456".to_string()));
    }

    #[test]
    fn test_roundtrip() {
        let original = Barcodes {
            ean13: Some("test".to_string()),
            ean8:  Some("other".to_string()),
        };
        let db: DbBarcodes = original.clone().into();
        let back: Barcodes = db.into();
        assert_eq!(original, back);
    }
}
