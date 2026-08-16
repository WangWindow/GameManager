#[test]
fn binary_name_matches_the_release_asset_contract() {
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_gamemanager"));
    assert_eq!(
        binary.file_name().and_then(|name| name.to_str()),
        Some("gamemanager")
    );
}
