use std::time::Duration;

use super::*;

const MISSING_DATA_DIR_MESSAGE: &str = "MEDIOSS_DATA_DIR is required: set it to the writable directory that will hold medioss.db and media/";
const REQUIRED_VARS: [(&str, &str); 3] = [
    ("MEDIOSS_DATA_DIR", "/data"),
    ("SESSION_SECRET", "setup-possession-secret"),
    ("MEDIOSS_ORIGIN", "https://medioss.example"),
];

fn vars(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let pairs = pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect::<Vec<_>>();
    move |key| {
        pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.clone())
    }
}

fn required_vars(overrides: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let pairs = REQUIRED_VARS
        .iter()
        .filter(|(name, _)| {
            !overrides
                .iter()
                .any(|(override_name, _)| override_name == name)
        })
        .chain(overrides.iter())
        .copied()
        .collect::<Vec<_>>();
    vars(&pairs)
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
fn whitespace_only_data_dir_is_an_error_with_the_documented_message() {
    let error = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "   ")])).unwrap_err();

    assert_eq!(error, ConfigError::MissingDataDir);
    assert_eq!(error.to_string(), MISSING_DATA_DIR_MESSAGE);
}

#[test]
fn port_defaults_to_3000() {
    let config = from_vars(&required_vars(&[])).unwrap();

    assert_eq!(config.port, 3000);
}

#[test]
fn port_override_is_respected() {
    let config = from_vars(&required_vars(&[("PORT", "8080")])).unwrap();

    assert_eq!(config.port, 8080);
}

#[test]
fn bind_host_defaults_to_all_interfaces() {
    let config = from_vars(&required_vars(&[])).unwrap();
    assert_eq!(config.bind_host.to_string(), "0.0.0.0");
}

#[test]
fn bind_host_override_is_respected() {
    let config = from_vars(&required_vars(&[("MEDIOSS_BIND", "127.0.0.1")])).unwrap();
    assert_eq!(config.bind_host.to_string(), "127.0.0.1");
}

#[test]
fn invalid_bind_host_names_the_variable() {
    let error = from_vars(&required_vars(&[("MEDIOSS_BIND", "localhost")])).unwrap_err();
    assert!(error.to_string().contains("MEDIOSS_BIND"));
}

#[test]
fn invalid_port_names_the_variable() {
    let error = from_vars(&required_vars(&[("PORT", "not-a-port")])).unwrap_err();

    assert!(error.to_string().contains("PORT"));
}

#[test]
fn static_dir_defaults_into_the_source_tree() {
    let config = from_vars(&required_vars(&[])).unwrap();

    assert_eq!(config.static_dir, PathBuf::from(DEFAULT_STATIC_DIR));
}

#[test]
fn static_dir_override_is_respected() {
    let config = from_vars(&required_vars(&[("MEDIOSS_STATIC_DIR", "/srv/ui")])).unwrap();

    assert_eq!(config.static_dir, PathBuf::from("/srv/ui"));
}

#[test]
fn missing_session_secret_names_the_variable() {
    let error = from_vars(&vars(&[
        ("MEDIOSS_DATA_DIR", "/data"),
        ("MEDIOSS_ORIGIN", "https://medioss.example"),
    ]))
    .unwrap_err();

    assert_eq!(error, ConfigError::MissingSessionSecret);
    assert!(error.to_string().contains("SESSION_SECRET"));
}

#[test]
fn blank_session_secret_names_the_variable() {
    let error = from_vars(&required_vars(&[("SESSION_SECRET", "   ")])).unwrap_err();

    assert_eq!(error, ConfigError::MissingSessionSecret);
    assert!(error.to_string().contains("SESSION_SECRET"));
}

#[test]
fn session_secret_is_trimmed_and_redacted_from_debug() {
    let config = from_vars(&required_vars(&[("SESSION_SECRET", "  private value  ")])).unwrap();

    assert_eq!(config.session_secret.expose(), "private value");
    assert!(!format!("{config:?}").contains("private value"));
}

#[test]
fn session_timeout_defaults_to_seven_days() {
    let config = from_vars(&required_vars(&[])).unwrap();

    assert_eq!(
        config.session_timeout,
        Duration::from_secs(7 * 24 * 60 * 60)
    );
}

#[test]
fn session_timeout_accepts_supported_units() {
    for (value, expected) in [
        ("12h", Duration::from_secs(12 * 60 * 60)),
        ("30m", Duration::from_secs(30 * 60)),
        ("45s", Duration::from_secs(45)),
    ] {
        let config = from_vars(&required_vars(&[("SESSION_TIMEOUT", value)])).unwrap();
        assert_eq!(config.session_timeout, expected, "value: {value}");
    }
}

#[test]
fn session_timeout_rejects_malformed_zero_and_overflowing_values() {
    for value in ["banana", "0s", "-1s", "18446744073709551616d"] {
        let error = from_vars(&required_vars(&[("SESSION_TIMEOUT", value)])).unwrap_err();
        assert_eq!(
            error,
            ConfigError::InvalidSessionTimeout {
                value: value.to_owned()
            }
        );
        assert!(error.to_string().contains("SESSION_TIMEOUT"));
    }
}

#[test]
fn missing_origin_names_the_variable() {
    let error = from_vars(&vars(&[
        ("MEDIOSS_DATA_DIR", "/data"),
        ("SESSION_SECRET", "setup-possession-secret"),
    ]))
    .unwrap_err();

    assert_eq!(error, ConfigError::MissingMediossOrigin);
    assert!(error.to_string().contains("MEDIOSS_ORIGIN"));
}

#[test]
fn origin_is_canonicalized_for_strict_comparison() {
    let config = from_vars(&required_vars(&[(
        "MEDIOSS_ORIGIN",
        "HTTPS://MEDIOSS.EXAMPLE:443/",
    )]))
    .unwrap();

    assert_eq!(config.medioss_origin, "https://medioss.example");
}

#[test]
fn origin_rejects_non_origins() {
    for value in [
        "*",
        "medioss.example",
        "ftp://medioss.example",
        "https://*.example.com",
        "https://medioss.example/path",
        "https://medioss.example//",
        "https://medioss.example/%2F",
        "https://medioss.example?query=yes",
        "https://medioss.example/#fragment",
        "https://user@medioss.example",
        "https://user:password@medioss.example",
        "https://@medioss.example",
        " https://medioss.example",
    ] {
        let error = from_vars(&required_vars(&[("MEDIOSS_ORIGIN", value)])).unwrap_err();
        assert_eq!(
            error,
            ConfigError::InvalidMediossOrigin {
                value: value.to_owned()
            },
            "value: {value}"
        );
        assert!(error.to_string().contains("MEDIOSS_ORIGIN"));
    }
}

#[test]
fn dev_origin_is_disabled_when_unset_or_blank() {
    assert_eq!(from_vars(&required_vars(&[])).unwrap().dev_origin, None);
    let config = from_vars(&required_vars(&[("MEDIOSS_DEV_ORIGIN", "  ")])).unwrap();
    assert_eq!(config.dev_origin, None);
}

#[test]
fn dev_origin_is_canonicalized() {
    let config = from_vars(&required_vars(&[(
        "MEDIOSS_DEV_ORIGIN",
        "HTTP://LOCALHOST:4321/",
    )]))
    .unwrap();

    assert_eq!(config.dev_origin.as_deref(), Some("http://localhost:4321"));
}

#[test]
fn dev_origin_rejects_non_origins() {
    for value in [
        "null",
        "*",
        "http://localhost:4321/path",
        " http://localhost:4321",
    ] {
        let error = from_vars(&required_vars(&[("MEDIOSS_DEV_ORIGIN", value)])).unwrap_err();
        assert_eq!(
            error,
            ConfigError::InvalidDevOrigin {
                value: value.to_owned()
            },
            "value: {value}"
        );
        assert!(error.to_string().contains("MEDIOSS_DEV_ORIGIN"));
    }
}

#[test]
fn session_cookie_secure_defaults_to_true() {
    let config = from_vars(&required_vars(&[])).unwrap();

    assert!(config.session_cookie_secure);
}

#[test]
fn session_cookie_secure_accepts_explicit_false() {
    let config = from_vars(&required_vars(&[("SESSION_COOKIE_SECURE", "false")])).unwrap();

    assert!(!config.session_cookie_secure);
}

#[test]
fn session_cookie_secure_rejects_non_literal_booleans() {
    for value in ["yes", "0", " true", "false ", "TRUE"] {
        let error = from_vars(&required_vars(&[("SESSION_COOKIE_SECURE", value)])).unwrap_err();
        assert_eq!(
            error,
            ConfigError::InvalidSessionCookieSecure {
                value: value.to_owned()
            },
            "value: {value}"
        );
        assert!(error.to_string().contains("SESSION_COOKIE_SECURE"));
    }
}

const UPLOAD_LIMIT_VARS: [&str; 3] = [
    "MEDIOSS_MAX_UPLOAD_BYTES",
    "MEDIOSS_MAX_SOUND_UPLOAD_BYTES",
    "MEDIOSS_MAX_COVER_BYTES",
];

#[test]
fn upload_limits_default_to_the_documented_sizes() {
    let config = from_vars(&required_vars(&[])).unwrap();

    assert_eq!(
        config.upload_limits,
        UploadLimits {
            meditation_audio_bytes: 104_857_600,
            sound_audio_bytes: 104_857_600,
            program_cover_bytes: 5_242_880,
        }
    );
}

#[test]
fn upload_limits_accept_valid_overrides_independently() {
    let config = from_vars(&required_vars(&[
        ("MEDIOSS_MAX_UPLOAD_BYTES", "209715200"),
        ("MEDIOSS_MAX_SOUND_UPLOAD_BYTES", " 1024 "),
        ("MEDIOSS_MAX_COVER_BYTES", "1"),
    ]))
    .unwrap();

    assert_eq!(config.upload_limits.meditation_audio_bytes, 209_715_200);
    assert_eq!(config.upload_limits.sound_audio_bytes, 1024);
    assert_eq!(config.upload_limits.program_cover_bytes, 1);
}

#[test]
fn upload_limit_accepts_the_ceiling_and_rejects_one_byte_above() {
    let ceiling = UPLOAD_LIMIT_CEILING_BYTES.to_string();
    let above = (UPLOAD_LIMIT_CEILING_BYTES + 1).to_string();

    for variable in UPLOAD_LIMIT_VARS {
        assert!(from_vars(&required_vars(&[(variable, &ceiling)])).is_ok());
        let error = from_vars(&required_vars(&[(variable, &above)])).unwrap_err();
        assert!(error.to_string().contains(variable), "{variable}");
    }
}

#[test]
fn upload_limit_rejects_zero_and_non_numeric_values_naming_the_variable() {
    for variable in UPLOAD_LIMIT_VARS {
        for value in ["0", "abc", "", "-5", "1.5", "100MB"] {
            let error = from_vars(&required_vars(&[(variable, value)])).unwrap_err();
            assert!(
                matches!(error, ConfigError::InvalidUploadLimit { variable: named, .. } if named == variable),
                "{variable}={value:?} gave {error}"
            );
        }
    }
}
