use super::*;

const MISSING_DATA_DIR_MESSAGE: &str = "MEDIOSS_DATA_DIR is required: set it to the writable directory that will hold medioss.db and media/";

fn vars(pairs: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    let pairs = pairs.to_vec();
    move |key| {
        pairs
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).to_owned())
    }
}

#[test]
fn missing_data_dir_is_an_error_with_the_documented_message() {
    let error = from_vars(&|_| None).unwrap_err();

    assert_eq!(error, ConfigError::MissingDataDir);
    assert_eq!(error.to_string(), MISSING_DATA_DIR_MESSAGE);
}

#[test]
fn empty_data_dir_is_an_error_with_the_documented_message() {
    let error = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "")])).unwrap_err();

    assert_eq!(error, ConfigError::MissingDataDir);
    assert_eq!(error.to_string(), MISSING_DATA_DIR_MESSAGE);
}

#[test]
fn port_defaults_to_3000() {
    let config = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "/data")])).unwrap();

    assert_eq!(config.port, 3000);
    assert_eq!(config.data_dir, PathBuf::from("/data"));
}

#[test]
fn port_override_is_respected() {
    let config = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "/data"), ("PORT", "8080")])).unwrap();

    assert_eq!(config.port, 8080);
}

#[test]
fn invalid_port_names_the_variable() {
    let error = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "/data"), ("PORT", "http")])).unwrap_err();

    assert!(error.to_string().contains("PORT"));
}

#[test]
fn static_dir_defaults_into_the_source_tree_and_is_overridable() {
    let default = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "/data")])).unwrap();
    assert_eq!(default.static_dir, PathBuf::from("src/apps/web-api/static"));

    let overridden = from_vars(&vars(&[
        ("MEDIOSS_DATA_DIR", "/data"),
        ("MEDIOSS_STATIC_DIR", "/srv/static"),
    ]))
    .unwrap();
    assert_eq!(overridden.static_dir, PathBuf::from("/srv/static"));
}
