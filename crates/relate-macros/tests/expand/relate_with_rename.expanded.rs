//! Relate derive with field rename expansion test.
use relate::Relate;
struct Source {
    id: String,
    name: String,
}
#[relate(Source)]
struct Target {
    #[relate(.id)]
    moysklad_id: String,
    name: String,
    #[relate(.name)]
    sync_name: String,
}
impl ::core::convert::From<Source> for Target {
    fn from(src: Source) -> Self {
        let __with_moysklad_id = src.id;
        let __with_sync_name = (src.name).clone();
        Self {
            moysklad_id: __with_moysklad_id,
            name: src.name.clone(),
            sync_name: __with_sync_name,
        }
    }
}
impl ::core::convert::From<&Source> for Target {
    fn from(src: &Source) -> Self {
        let __with_moysklad_id = (src.id).clone();
        let __with_sync_name = (src.name).clone();
        Self {
            moysklad_id: __with_moysklad_id,
            name: src.name.clone(),
            sync_name: __with_sync_name,
        }
    }
}
fn main() {}
