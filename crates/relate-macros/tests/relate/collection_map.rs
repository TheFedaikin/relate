//! Tests for collection mapping in relate_structs!

use relate::relate_structs;

// Test collection map with cloned modifier
mod collection_map_cloned {
    use super::*;

    #[derive(Debug, Clone)]
    struct Item {
        id:    i32,
        #[allow(dead_code)]
        _name: String,
    }

    #[derive(Debug, Clone)]
    struct Container {
        items: Vec<Item>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct ContainerDto {
        items: Vec<i32>,
    }

    relate_structs! {
        Container ~> ContainerDto {
            items: with = [_.id], cloned;
        }
    }

    #[test]
    fn test_collection_map_with_cloned() {
        let container = Container {
            items: vec![
                Item {
                    id:    1,
                    _name: "First".to_string(),
                },
                Item {
                    id:    2,
                    _name: "Second".to_string(),
                },
            ],
        };
        let dto: ContainerDto = container.into();
        assert_eq!(dto.items, vec![1, 2]);
    }

    #[test]
    fn test_from_ref_with_cloned() {
        let container = Container {
            items: vec![Item {
                id:    42,
                _name: "Test".to_string(),
            }],
        };
        let dto: ContainerDto = (&container).into();
        assert_eq!(dto.items, vec![42]);
        // Original still usable
        assert_eq!(container.items.len(), 1);
    }
}

mod basic_collection_map {
    use super::*;

    #[derive(Debug, Clone)]
    struct Variant {
        id:    String,
        #[allow(dead_code)]
        _name: String,
    }

    #[derive(Debug, Clone)]
    struct ProductWithVariants {
        id:       String,
        variants: Vec<Variant>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Product {
        id:       String,
        variants: Vec<String>,
    }

    relate_structs! {
        ProductWithVariants ~> Product {
            id;
            variants: with = [_.id.clone()];
        }
    }

    #[test]
    fn test_collection_map() {
        let product = ProductWithVariants {
            id:       "prod1".to_string(),
            variants: vec![
                Variant {
                    id:    "v1".to_string(),
                    _name: "Small".to_string(),
                },
                Variant {
                    id:    "v2".to_string(),
                    _name: "Large".to_string(),
                },
            ],
        };
        let result: Product = product.into();

        assert_eq!(result.id, "prod1");
        assert_eq!(result.variants, vec!["v1", "v2"]);
    }

    #[test]
    fn test_empty_collection() {
        let product = ProductWithVariants {
            id:       "prod2".to_string(),
            variants: vec![],
        };
        let result: Product = product.into();
        assert!(result.variants.is_empty());
    }
}
