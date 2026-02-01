//! Tests for collection mapping in Relate derive.

use relate::Relate;

#[derive(Debug, Clone)]
struct Variant {
    id:    String,
    #[allow(dead_code)]
    _name: String,
}

#[derive(Debug, Clone)]
struct ProductWithVariants {
    id:       String,
    name:     String,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(ProductWithVariants)]
struct Product {
    id:       String,
    name:     String,
    #[relate([.id.clone()])]
    variants: Vec<String>,
}

#[test]
fn test_collection_map() {
    let product = ProductWithVariants {
        id:       "prod-1".to_string(),
        name:     "Test Product".to_string(),
        variants: vec![
            Variant {
                id:    "var-1".to_string(),
                _name: "Small".to_string(),
            },
            Variant {
                id:    "var-2".to_string(),
                _name: "Large".to_string(),
            },
        ],
    };

    let result: Product = product.into();

    assert_eq!(result.id, "prod-1");
    assert_eq!(result.name, "Test Product");
    assert_eq!(
        result.variants,
        vec!["var-1".to_string(), "var-2".to_string()]
    );
}

// Test collection map from reference
#[test]
fn test_collection_map_from_ref() {
    let product = ProductWithVariants {
        id:       "prod-2".to_string(),
        name:     "Another".to_string(),
        variants: vec![Variant {
            id:    "var-x".to_string(),
            _name: "X".to_string(),
        }],
    };

    let result: Product = (&product).into();

    assert_eq!(result.variants, vec!["var-x".to_string()]);

    // Original still usable
    assert_eq!(product.variants[0].id, "var-x");
}

// Test collection map to computed value
#[derive(Debug, Clone)]
struct Item {
    name: String,
}

#[derive(Debug, Clone)]
struct ItemCollection {
    items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(ItemCollection)]
struct LengthCollection {
    #[relate([.name.len()])]
    items: Vec<usize>,
}

#[test]
fn test_collection_map_method_chain() {
    let collection = ItemCollection {
        items: vec![
            Item {
                name: "a".to_string(),
            },
            Item {
                name: "abc".to_string(),
            },
            Item {
                name: "abcde".to_string(),
            },
        ],
    };

    let result: LengthCollection = collection.into();

    assert_eq!(result.items, vec![1, 3, 5]);
}
