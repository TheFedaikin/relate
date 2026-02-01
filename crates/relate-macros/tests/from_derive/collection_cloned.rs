//! Tests for collection syntax with clone modes and Into::into conversion.

use relate::Relate;

// =============================================================================
// Basic Collection with Cloned Mode
// =============================================================================

mod basic_collection {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct SourceItem {
        id: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TargetItem {
        id: i32,
    }

    impl From<SourceItem> for TargetItem {
        fn from(s: SourceItem) -> Self { TargetItem { id: s.id } }
    }

    #[derive(Debug, Clone)]
    struct Source {
        items: Vec<SourceItem>,
    }

    // With cloned mode, [_] generates:
    // src.items.iter().cloned().map(Into::into).collect()
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct Target {
        #[relate([_])]
        items: Vec<TargetItem>,
    }

    #[test]
    fn test_collection_with_cloned_and_into() {
        let source = Source {
            items: vec![
                SourceItem { id: 1 },
                SourceItem { id: 2 },
                SourceItem { id: 3 },
            ],
        };

        let target: Target = source.into();

        assert_eq!(target.items.len(), 3);
        assert_eq!(target.items[0].id, 1);
        assert_eq!(target.items[1].id, 2);
        assert_eq!(target.items[2].id, 3);
    }

    #[test]
    fn test_collection_from_ref() {
        let source = Source {
            items: vec![SourceItem { id: 10 }, SourceItem { id: 20 }],
        };

        let target: Target = (&source).into();

        assert_eq!(target.items.len(), 2);
        // source still available
        assert_eq!(source.items.len(), 2);
    }
}

// =============================================================================
// Collection with Field Access
// =============================================================================

mod collection_field_access {
    use super::*;

    #[derive(Debug, Clone)]
    struct Item {
        id:   i32,
        name: String,
    }

    #[derive(Debug, Clone)]
    struct Source {
        items: Vec<Item>,
    }

    // Extract just IDs from items using [_.field] syntax
    // This becomes: src.items.iter().map(|__item| __item.id).collect()
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct IdsOnly {
        #[relate([_.id])]
        items: Vec<i32>,
    }

    #[test]
    fn test_collection_extract_field() {
        let source = Source {
            items: vec![
                Item {
                    id:   1,
                    name: "a".to_string(),
                },
                Item {
                    id:   2,
                    name: "b".to_string(),
                },
            ],
        };

        let ids: IdsOnly = source.into();

        assert_eq!(ids.items, vec![1, 2]);
    }

    // Extract names with transform
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct NamesUpper {
        #[relate([_.name.to_uppercase()])]
        items: Vec<String>,
    }

    #[test]
    fn test_collection_field_with_method() {
        let source = Source {
            items: vec![
                Item {
                    id:   1,
                    name: "hello".to_string(),
                },
                Item {
                    id:   2,
                    name: "world".to_string(),
                },
            ],
        };

        let names: NamesUpper = source.into();

        assert_eq!(names.items, vec!["HELLO".to_string(), "WORLD".to_string()]);
    }
}

// =============================================================================
// Nested Collection Access (via from = expr)
// =============================================================================

mod nested_collection {
    use super::*;

    #[derive(Debug, Clone)]
    struct Container {
        data: DataHolder,
    }

    #[derive(Debug, Clone)]
    struct DataHolder {
        tags: Vec<String>,
    }

    // For nested access to a collection, use with = expr
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Container, cloned)]
    struct Flat {
        #[relate(with = .data.tags.clone())]
        tags: Vec<String>,
    }

    #[test]
    fn test_nested_collection_access() {
        let container = Container {
            data: DataHolder {
                tags: vec!["a".to_string(), "b".to_string()],
            },
        };

        let flat: Flat = container.into();

        assert_eq!(flat.tags, vec!["a".to_string(), "b".to_string()]);
    }
}

// =============================================================================
// Collection Same Type
// =============================================================================

mod same_type_collection {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        names: Vec<String>,
    }

    // Same type collection - [_] with cloned mode just clones and applies
    // Into::into (which is a no-op for same types)
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct Target {
        #[relate([_])]
        names: Vec<String>,
    }

    #[test]
    fn test_same_type_collection() {
        let source = Source {
            names: vec!["alice".to_string(), "bob".to_string()],
        };

        let target: Target = source.into();

        assert_eq!(target.names, vec!["alice".to_string(), "bob".to_string()]);
    }
}

// =============================================================================
// Collection with Derive Relate Items
// =============================================================================

mod collection_with_derive_items {
    use super::*;

    #[derive(Debug, Clone)]
    struct SourceVariant {
        sku:   String,
        price: i32,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(SourceVariant)]
    struct TargetVariant {
        sku:   String,
        price: i32,
    }

    #[derive(Debug, Clone)]
    struct SourceProduct {
        name:     String,
        variants: Vec<SourceVariant>,
    }

    // Collection of items that themselves implement From
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(SourceProduct, cloned)]
    struct TargetProduct {
        name:     String,
        #[relate([_])]
        variants: Vec<TargetVariant>,
    }

    #[test]
    fn test_collection_of_derived_items() {
        let source = SourceProduct {
            name:     "Widget".to_string(),
            variants: vec![
                SourceVariant {
                    sku:   "W-001".to_string(),
                    price: 100,
                },
                SourceVariant {
                    sku:   "W-002".to_string(),
                    price: 150,
                },
            ],
        };

        let target: TargetProduct = source.into();

        assert_eq!(target.name, "Widget");
        assert_eq!(target.variants.len(), 2);
        assert_eq!(target.variants[0].sku, "W-001");
        assert_eq!(target.variants[1].price, 150);
    }
}

// =============================================================================
// Empty Collections
// =============================================================================

mod empty_collections {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        items: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct Target {
        #[relate([_])]
        items: Vec<String>,
    }

    #[test]
    fn test_empty_collection() {
        let source = Source { items: vec![] };

        let target: Target = source.into();

        assert!(target.items.is_empty());
    }
}

// =============================================================================
// Collection with Complex Transforms (using from = expr)
// =============================================================================

mod complex_transforms {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        numbers: Vec<i32>,
    }

    // Filter and transform using with = expr
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct Filtered {
        #[relate(with = .numbers.iter().filter(|n| **n > 0).map(|n| n.to_string()).collect())]
        numbers: Vec<String>,
    }

    #[test]
    fn test_complex_collection_transform() {
        let source = Source {
            numbers: vec![-1, 0, 1, 2, -3, 4],
        };

        let target: Filtered = source.into();

        assert_eq!(target.numbers, vec!["1", "2", "4"]);
    }
}

// =============================================================================
// Multiple Collections
// =============================================================================

mod multiple_collections {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        names: Vec<String>,
        ages:  Vec<i32>,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct Target {
        #[relate([_])]
        names: Vec<String>,
        #[relate([_])]
        ages:  Vec<i32>,
    }

    #[test]
    fn test_multiple_collections() {
        let source = Source {
            names: vec!["alice".to_string(), "bob".to_string()],
            ages:  vec![30, 25],
        };

        let target: Target = source.into();

        assert_eq!(target.names, vec!["alice".to_string(), "bob".to_string()]);
        assert_eq!(target.ages, vec![30, 25]);
    }
}

// =============================================================================
// Collection Without Cloned Mode (Auto)
// =============================================================================

mod auto_mode_collection {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        items: Vec<String>,
    }

    // Without cloned mode, uses regular .iter().map()
    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source)]
    struct Target {
        #[relate([_.clone()])]
        items: Vec<String>,
    }

    #[test]
    fn test_collection_auto_mode() {
        let source = Source {
            items: vec!["x".to_string(), "y".to_string()],
        };

        let target: Target = source.into();

        assert_eq!(target.items, vec!["x".to_string(), "y".to_string()]);
    }
}

// =============================================================================
// Collection with Rename
// =============================================================================

mod collection_with_rename {
    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        old_items: Vec<i32>,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct Target {
        #[relate(with = .old_items.clone())]
        new_items: Vec<i32>,
    }

    #[test]
    fn test_collection_with_field_rename() {
        let source = Source {
            old_items: vec![1, 2, 3],
        };

        let target: Target = source.into();

        assert_eq!(target.new_items, vec![1, 2, 3]);
    }
}

// =============================================================================
// HashSet Collection
// =============================================================================

mod hashset_collection {
    use std::collections::HashSet;

    use super::*;

    #[derive(Debug, Clone)]
    struct Source {
        tags: HashSet<String>,
    }

    #[derive(Debug, Clone, PartialEq, Relate)]
    #[relate(Source, cloned)]
    struct Target {
        #[relate([_])]
        tags: HashSet<String>,
    }

    #[test]
    fn test_hashset_collection() {
        let mut tags = HashSet::new();
        tags.insert("rust".to_string());
        tags.insert("macro".to_string());

        let source = Source { tags };
        let target: Target = source.into();

        assert!(target.tags.contains("rust"));
        assert!(target.tags.contains("macro"));
    }
}
