//! Test: Auto TryFrom inference compiles correctly

use relate::Relate;

// Basic auto TryFrom via `?` operator
struct RawConfig {
    port: String,
    host: String,
}

#[derive(Relate)]
#[relate(RawConfig)]
struct Config {
    #[relate(_.parse()?)]
    port: u16,
    host: String,
}

// Multiple fallible fields
struct RawData {
    x: String,
    y: String,
}

#[derive(Relate)]
#[relate(RawData)]
struct ParsedData {
    #[relate(_.parse()?)]
    x: i32,
    #[relate(_.parse()?)]
    y: i32,
}

// Explicit error type
#[derive(Debug)]
struct MyError(String);

impl From<std::num::ParseIntError> for MyError {
    fn from(e: std::num::ParseIntError) -> Self {
        MyError(e.to_string())
    }
}

struct RawValue {
    val: String,
}

#[derive(Relate)]
#[relate(RawValue, error = MyError)]
struct ParsedValue {
    #[relate(_.parse()?)]
    val: i32,
}

// With clone mode
struct ClonedSource {
    num: String,
}

#[derive(Relate)]
#[relate(ClonedSource, cloned)]
struct ClonedTarget {
    #[relate(_.parse()?)]
    num: i32,
}

fn main() {
    // Auto TryFrom
    let raw = RawConfig {
        port: "8080".to_string(),
        host: "localhost".to_string(),
    };
    let _config: Result<Config, _> = raw.try_into();

    // Multiple fallible
    let raw_data = RawData {
        x: "1".to_string(),
        y: "2".to_string(),
    };
    let _parsed: Result<ParsedData, _> = raw_data.try_into();

    // Explicit error
    let raw_val = RawValue {
        val: "42".to_string(),
    };
    let _val: Result<ParsedValue, MyError> = raw_val.try_into();

    // With clone mode
    let cloned_src = ClonedSource {
        num: "100".to_string(),
    };
    let _cloned: Result<ClonedTarget, _> = cloned_src.try_into();
}
