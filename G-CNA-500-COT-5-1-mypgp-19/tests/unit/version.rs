use my_pgp_lib::VERSION;

#[test]
fn test_version_is_not_empty() {
    assert!(
        !VERSION.is_empty(),
        "VERSION comes from Cargo.toml and must never be empty"
    );
}
