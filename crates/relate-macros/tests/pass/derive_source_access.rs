//! Test: Source access syntax `.path.field` and `_.method()` compiles correctly

use relate::Relate;

struct Wrapper {
    inner: Inner,
}

struct Inner {
    name:  String,
    value: i32,
}

#[derive(Relate)]
#[relate(Wrapper)]
struct Flat {
    #[relate(.inner.name)]
    name: String,
    #[relate(.inner.value)]
    value: i32,
}

// Deep nesting with `.path` syntax
struct Level1 {
    level2: Level2,
}

struct Level2 {
    level3: Level3,
}

struct Level3 {
    data: String,
}

#[derive(Relate)]
#[relate(Level1)]
struct DeepFlat {
    #[relate(.level2.level3.data)]
    data: String,
}

// Using `_` as field name placeholder in path
struct Deep1 {
    deep2: Deep2,
}

struct Deep2 {
    name: String,
}

#[derive(Relate)]
#[relate(Deep1)]
struct DeepWithPlaceholder {
    // .deep2._ means src.deep2.{field_name} = src.deep2.name
    #[relate(.deep2._)]
    name: String,
}

// With method calls using `_` placeholder
struct Container {
    items: Vec<i32>,
}

#[derive(Relate)]
#[relate(Container)]
struct Summary {
    #[relate(_.len())]
    items: usize,
}

// Method on same-named field
struct Source {
    name: String,
    id:   Option<i32>,
}

#[derive(Relate)]
#[relate(Source)]
struct Target {
    #[relate(_.to_uppercase())]
    name: String,
    #[relate(_.unwrap_or(0))]
    id: i32,
}

// Mixed `.path._` pattern
struct Outer {
    inner: InnerData,
}

struct InnerData {
    text: String,
}

#[derive(Relate)]
#[relate(Outer)]
struct ExtractText {
    #[relate(.inner._)]
    text: String,
}

fn main() {
    // Basic source access
    let wrapper = Wrapper {
        inner: Inner {
            name:  "test".to_string(),
            value: 42,
        },
    };
    let _flat: Flat = wrapper.into();

    // Deep nesting with .path syntax
    let l1 = Level1 {
        level2: Level2 {
            level3: Level3 {
                data: "deep".to_string(),
            },
        },
    };
    let _deep: DeepFlat = l1.into();

    // Using _ as field name placeholder in path
    let d1 = Deep1 {
        deep2: Deep2 {
            name: "placeholder".to_string(),
        },
    };
    let _deep_placeholder: DeepWithPlaceholder = d1.into();

    // With method using _ placeholder
    let container = Container { items: vec![1, 2, 3] };
    let _summary: Summary = container.into();

    // Method on same-named field
    let source = Source {
        name: "hello".to_string(),
        id:   Some(42),
    };
    let _target: Target = source.into();

    // Mixed .path._ pattern
    let outer = Outer {
        inner: InnerData {
            text: "extracted".to_string(),
        },
    };
    let _extract: ExtractText = outer.into();
}
