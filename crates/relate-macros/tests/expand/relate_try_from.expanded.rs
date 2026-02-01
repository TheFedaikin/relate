//! TryFrom expansion test - shows how fallible expressions generate TryFrom.
use relate::Relate;
struct Source {
    port: String,
    host: String,
}
#[relate(Source)]
struct Target {
    #[relate(_.parse()?)]
    port: u16,
    host: String,
}
impl ::core::convert::TryFrom<Source> for Target {
    type Error = ::relate::ConversionError;
    fn try_from(src: Source) -> ::core::result::Result<Self, Self::Error> {
        let __with_port = src.port.parse()?;
        ::core::result::Result::Ok(Self {
            port: __with_port,
            host: src.host,
        })
    }
}
impl ::core::convert::TryFrom<&Source> for Target {
    type Error = ::relate::ConversionError;
    fn try_from(src: &Source) -> ::core::result::Result<Self, Self::Error> {
        let __with_port = src.port.parse()?;
        ::core::result::Result::Ok(Self {
            port: __with_port,
            host: src.host.clone(),
        })
    }
}
fn main() {}
