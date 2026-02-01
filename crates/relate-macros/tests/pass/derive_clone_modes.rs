//! Test: Clone modes (cloned/move) compile correctly

use relate::Relate;

#[derive(Clone)]
struct Source {
    name:  String,
    value: i32,
}

// Struct-level cloned
#[derive(Relate)]
#[relate(Source, cloned)]
struct TargetCloned {
    name:  String,
    value: i32,
}

// Struct-level move
#[derive(Relate)]
#[relate(Source, move)]
struct TargetMove {
    name:  String,
    value: i32,
}

// Field-level override
#[derive(Relate)]
#[relate(Source, cloned)]
struct TargetMixed {
    name: String,
    #[relate(move)]
    value: i32,
}

// Rename with clone mode
#[derive(Clone)]
struct SourceRename {
    old_name: String,
}

#[derive(Relate)]
#[relate(SourceRename)]
struct TargetRename {
    #[relate(.old_name, cloned)]
    new_name: String,
}

// With bidirectional
#[derive(Clone, PartialEq)]
struct BidirSource {
    data: String,
}

#[derive(Relate, Clone, PartialEq)]
#[relate(BidirSource, both, cloned)]
struct BidirTarget {
    data: String,
}

fn main() {
    let source = Source {
        name:  "test".to_string(),
        value: 42,
    };

    let _cloned: TargetCloned = source.clone().into();
    let _moved: TargetMove = source.clone().into();
    let _mixed: TargetMixed = source.into();

    let rename_src = SourceRename {
        old_name: "rename".to_string(),
    };
    let _renamed: TargetRename = rename_src.into();

    let bidir = BidirSource {
        data: "bidir".to_string(),
    };
    let target: BidirTarget = bidir.clone().into();
    let _back: BidirSource = target.into();
}
