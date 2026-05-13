use clipstash_server::auth::AuthConfig;

fn config() -> AuthConfig {
    AuthConfig {
        username: "admin".into(),
        password: "secret".into(),
        api_key: "test-api-key".into(),
    }
}

#[test]
fn verify_password_correct() {
    assert!(config().verify_password("admin", "secret"));
}

#[test]
fn verify_password_wrong_user() {
    assert!(!config().verify_password("other", "secret"));
}

#[test]
fn verify_password_wrong_password() {
    assert!(!config().verify_password("admin", "wrong"));
}

#[test]
fn verify_password_both_wrong() {
    assert!(!config().verify_password("other", "wrong"));
}

#[test]
fn verify_api_key_correct() {
    assert!(config().verify_api_key("test-api-key"));
}

#[test]
fn verify_api_key_wrong() {
    assert!(!config().verify_api_key("bad-key"));
}

#[test]
fn verify_api_key_empty() {
    assert!(!config().verify_api_key(""));
}

#[test]
fn verify_password_prefix_not_accepted() {
    // "secret" is a prefix of "secretXYZ" — must not match
    assert!(!config().verify_password("admin", "secretXYZ"));
}

#[test]
fn verify_api_key_prefix_not_accepted() {
    assert!(!config().verify_api_key("test-api-key-extra"));
}
