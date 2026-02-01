//! Relate derive with transforms expansion test.
use relate::Relate;
struct Source {
    id: Option<i32>,
    name: String,
    encrypted: bool,
    value: String,
}
#[relate(Source)]
struct Target {
    #[relate(_.unwrap_or(0))]
    id: i32,
    name: String,
    #[relate(default = None)]
    extra: Option<String>,
    #[relate(with = if.encrypted{None}else{Some(.value.clone())})]
    value: Option<String>,
    encrypted: bool,
}
impl ::core::convert::From<Source> for Target {
    fn from(src: Source) -> Self {
        let __with_id = src.id.unwrap_or(0);
        let __with_value = if src.encrypted { None } else { Some(src.value.clone()) };
        Self {
            id: __with_id,
            name: src.name,
            extra: None,
            value: __with_value,
            encrypted: src.encrypted,
        }
    }
}
impl ::core::convert::From<&Source> for Target {
    fn from(src: &Source) -> Self {
        let __with_id = src.id.unwrap_or(0);
        let __with_value = if src.encrypted { None } else { Some(src.value.clone()) };
        Self {
            id: __with_id,
            name: src.name.clone(),
            extra: None,
            value: __with_value,
            encrypted: src.encrypted.clone(),
        }
    }
}
fn main() {}
