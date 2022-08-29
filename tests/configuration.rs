const DEFAULT_CONFIG: &'static str = include_str!("../assets/default_config.toml");

#[test]
fn default_config_deserializes() {
    let toml = toml::from_str::<toml::Value>(DEFAULT_CONFIG);
    assert!(toml.is_ok(), "Not ok: {:?}", toml.unwrap_err());
}
