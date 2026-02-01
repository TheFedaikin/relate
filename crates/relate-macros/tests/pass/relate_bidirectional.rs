//! Bidirectional relate_structs! - should compile.

use relate::relate_structs;

#[derive(Debug, Clone)]
struct ApiModel {
    id:   i32,
    name: String,
}

#[derive(Debug, Clone)]
struct DbModel {
    id:   i32,
    name: String,
}

relate_structs! {
    ApiModel ~ DbModel {
        id;
        name;
    }
}

fn main() {
    let api = ApiModel {
        id:   1,
        name: "test".to_string(),
    };
    let db: DbModel = api.into();

    let api2: ApiModel = db.into();
    assert_eq!(api2.id, 1);
}
