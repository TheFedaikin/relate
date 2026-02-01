//! Tests for `with = expr` syntax that accesses source struct fields.

use relate::Relate;

#[derive(Debug, Clone)]
struct Setting {
    name:      String,
    value:     String,
    encrypted: bool,
}

impl Setting {
    fn parsed_value(&self) -> Result<String, ()> { Ok(self.value.clone()) }
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(Setting)]
struct SettingResponse {
    name:      String,
    #[relate(with = if .encrypted { None } else { .parsed_value().ok() })]
    value:     Option<String>,
    encrypted: bool,
}

#[test]
fn test_with_expr_conditional() {
    let setting = Setting {
        name:      "api_key".to_string(),
        value:     "secret123".to_string(),
        encrypted: true,
    };

    let response: SettingResponse = setting.into();

    assert_eq!(response.name, "api_key");
    assert_eq!(response.value, None); // encrypted, so value is hidden
    assert!(response.encrypted);
}

#[test]
fn test_with_expr_not_encrypted() {
    let setting = Setting {
        name:      "theme".to_string(),
        value:     "dark".to_string(),
        encrypted: false,
    };

    let response: SettingResponse = setting.into();

    assert_eq!(response.name, "theme");
    assert_eq!(response.value, Some("dark".to_string()));
    assert!(!response.encrypted);
}

#[test]
fn test_with_expr_from_ref() {
    let setting = Setting {
        name:      "color".to_string(),
        value:     "blue".to_string(),
        encrypted: false,
    };

    let response: SettingResponse = (&setting).into();

    assert_eq!(response.value, Some("blue".to_string()));
    // Source still available
    assert_eq!(setting.value, "blue");
}

// Test with more complex expression
#[derive(Debug, Clone)]
struct User {
    first_name: String,
    last_name:  String,
    age:        u32,
}

#[derive(Debug, Clone, PartialEq, Relate)]
#[relate(User)]
struct UserSummary {
    #[relate(with = format!("{} {}", .first_name, .last_name))]
    full_name: String,
    #[relate(with = .age >= 18)]
    is_adult:  bool,
}

#[test]
fn test_with_expr_computed_fields() {
    let user = User {
        first_name: "John".to_string(),
        last_name:  "Doe".to_string(),
        age:        25,
    };

    let summary: UserSummary = user.into();

    assert_eq!(summary.full_name, "John Doe");
    assert!(summary.is_adult);
}

#[test]
fn test_with_expr_minor() {
    let user = User {
        first_name: "Jane".to_string(),
        last_name:  "Smith".to_string(),
        age:        16,
    };

    let summary: UserSummary = user.into();

    assert_eq!(summary.full_name, "Jane Smith");
    assert!(!summary.is_adult);
}
