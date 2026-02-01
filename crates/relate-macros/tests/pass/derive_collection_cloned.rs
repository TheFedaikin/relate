//! Test: Collection syntax with cloned mode compiles correctly

use relate::Relate;

// Basic collection with Into
#[derive(Clone)]
struct SourceItem {
    id: i32,
}

struct TargetItem {
    id: i32,
}

impl From<SourceItem> for TargetItem {
    fn from(s: SourceItem) -> Self {
        TargetItem { id: s.id }
    }
}

struct SourceList {
    items: Vec<SourceItem>,
}

#[derive(Relate)]
#[relate(SourceList, cloned)]
struct TargetList {
    #[relate([_])]
    items: Vec<TargetItem>,
}

// Collection with field access
#[derive(Clone)]
struct ItemWithName {
    name: String,
}

struct SourceItems {
    items: Vec<ItemWithName>,
}

#[derive(Relate)]
#[relate(SourceItems, cloned)]
struct ExtractedNames {
    #[relate([_.name])]
    items: Vec<String>,
}

// Same type collection (no Into needed)
struct SameSource {
    values: Vec<i32>,
}

#[derive(Relate)]
#[relate(SameSource, cloned)]
struct SameTarget {
    #[relate([_])]
    values: Vec<i32>,
}

fn main() {
    // Basic with Into
    let source = SourceList {
        items: vec![SourceItem { id: 1 }, SourceItem { id: 2 }],
    };
    let _target: TargetList = source.into();

    // Field access
    let items = SourceItems {
        items: vec![
            ItemWithName { name: "a".to_string() },
            ItemWithName { name: "b".to_string() },
        ],
    };
    let _names: ExtractedNames = items.into();

    // Same type
    let same = SameSource { values: vec![1, 2, 3] };
    let _same_target: SameTarget = same.into();
}
