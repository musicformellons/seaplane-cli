//! Config handles loading of, and updating the Context from, a configuration file.
//!
//! The config will look in several pre-determined (platform specific) locations. If a valid
//! configuration file is found, it's values are loaded. Note that later layers may override values
//! from previous layers.
//!
//! - System configuration files (currently none are defined)
//! - User configuration files
//!   - Linux
//!     - `$XDG_CONFIG_HOME/seaplane/`
//!     - `$HOME/.config/seaplane/`
//!     - `$HOME/.seaplane/`
//!   - macOS
//!     - `$HOME/Library/ApplicationSupport/io.Seaplane.seaplane/`
//!     - `$HOME/.config/seaplane/`
//!     - `$HOME/.seaplane/`
//!   - Windows
//!     - `%RoamingAppData%/Seaplane/seaplane/config/`
//!     - `$HOME/.config/seaplane/`
//!     - `$HOME/.seaplane/`
//! - The CLI's `--config` flag
//!
//! Note the CLI also provides a `--no-override` flag that prevents later configuration files from
//! overriding previously discovered configuration layers. In this case the final layer "wins" and
//! all previous layers are ignored. i.e. using `--config` will cause only that CLI provided
//! configuration to be considered and not any of those in the filesystem.
//!
//! See also the CONFIGURATION_SPEC.md in this repository

use std::{
    fs,
    path::{Path, PathBuf},
};

use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    cli::{CliCommand, SeaplaneInit},
    context::Ctx,
    error::{CliError, CliErrorKind, Result},
    fs::{conf_dirs, AtomicFile, FromDisk, ToDisk},
    printer::ColorChoice,
};

static SEAPLANE_CONFIG_FILE: &str = "seaplane.toml";

/// Extends a configuration instance with overriding config
pub trait ExtendConfig {
    fn extend(&mut self, other: &Self);
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawConfig {
    #[serde(skip)]
    pub loaded_from: Vec<PathBuf>,

    // Used to signal we already found a valid config and to warn the user we will be overriding
    #[serde(skip)]
    found: bool,

    /// Did we run initialization automatically or not on startup?
    #[serde(skip)]
    pub did_init: bool,

    #[serde(default)]
    pub seaplane: RawSeaplaneConfig,

    #[serde(default)]
    pub account: RawAccountConfig,

    #[serde(default)]
    pub api: RawApiConfig,

    #[serde(default, skip_serializing_if = "RawDangerZoneConfig::is_empty")]
    pub danger_zone: RawDangerZoneConfig,
}

impl RawConfig {
    /// Loads the Raw configuration file (not de-conflicted with the CLI or ENV yet)
    ///
    /// Loads configs from all platform specific locations, overriding values at each step
    pub fn load_all() -> Result<Self> {
        let mut cfg = RawConfig::default();

        for dir in conf_dirs() {
            let maybe_file = dir.join(SEAPLANE_CONFIG_FILE);

            let new_cfg = match RawConfig::load(&maybe_file) {
                Ok(cfg) => cfg,
                Err(e) => {
                    if e.kind() == &CliErrorKind::MissingPath {
                        continue;
                    }
                    return Err(e);
                }
            };

            if cfg.found {
                cli_warnln!("overriding previous configuration options with {maybe_file:?}");
                cli_warn!("(hint: use ");
                cli_warn!(@Green, "--verbose ");
                cli_warnln!(@noprefix, "for more info)");
            }

            cfg.update(new_cfg)?;
            cfg.found = true;
        }

        if !cfg.found {
            let mut ctx = Ctx::default();
            ctx.internal_run = true;
            SeaplaneInit.run(&mut ctx)?;
            cfg.did_init = true;
        }

        Ok(cfg)
    }

    fn update(&mut self, new_cfg: RawConfig) -> Result<()> {
        // TODO: as we get more keys and tables we'll need a better way to do this
        if let Some(key) = new_cfg.account.api_key {
            self.account.api_key = Some(key);
        }
        if let Some(choice) = new_cfg.seaplane.color {
            self.seaplane.color = Some(choice);
        }
        if let Some(registry) = new_cfg.seaplane.default_registry_url {
            self.seaplane.default_registry_url = Some(registry);
        }
        if let Some(url) = new_cfg.api.compute_url {
            self.api.compute_url = Some(url);
        }
        if let Some(url) = new_cfg.api.identity_url {
            self.api.identity_url = Some(url);
        }
        if let Some(url) = new_cfg.api.metadata_url {
            self.api.metadata_url = Some(url);
        }
        if let Some(url) = new_cfg.api.locks_url {
            self.api.locks_url = Some(url);
        }
        #[cfg(feature = "allow_insecure_urls")]
        {
            self.danger_zone.allow_insecure_urls = new_cfg.danger_zone.allow_insecure_urls;
        }
        #[cfg(feature = "allow_invalid_certs")]
        {
            self.danger_zone.allow_invalid_certs = new_cfg.danger_zone.allow_invalid_certs;
        }
        self.loaded_from.extend(new_cfg.loaded_from);
        Ok(())
    }
}

impl FromDisk for RawConfig {
    fn set_loaded_from<P: AsRef<Path>>(&mut self, p: P) {
        self.loaded_from.push(p.as_ref().into());
    }

    fn loaded_from(&self) -> Option<&Path> { self.loaded_from.get(0).map(|p| &**p) }

    fn load<P: AsRef<Path>>(p: P) -> Result<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let path = p.as_ref();

        cli_traceln!("Looking for configuration file at {path:?}");
        if !path.exists() {
            return Err(CliErrorKind::MissingPath.into_err());
        }

        cli_traceln!("Found configuration file {path:?}");
        let mut cfg: RawConfig = toml::from_str(&fs::read_to_string(&p)?)?;
        cfg.set_loaded_from(p);
        Ok(cfg)
    }
}

impl ToDisk for RawConfig {
    fn persist(&self) -> Result<()>
    where
        Self: Sized + Serialize,
    {
        if let Some(path) = self.loaded_from.get(0) {
            let file = AtomicFile::new(path)?;
            let toml_str = toml::to_string_pretty(self)?;

            // TODO: make atomic so that we don't lose or corrupt data
            // TODO: long term consider something like SQLite
            fs::write(file.temp_path(), toml_str).map_err(CliError::from)
        } else {
            Err(CliErrorKind::MissingPath.into_err())
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawSeaplaneConfig {
    /// Whether to color output or not
    #[serde(default)]
    pub color: Option<ColorChoice>,

    /// The default container image registry to infer if not provided
    #[serde(default)]
    pub default_registry_url: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawAccountConfig {
    /// The user's API key
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawApiConfig {
    /// The URL of Compute Service
    #[serde(default)]
    pub compute_url: Option<Url>,

    /// The URL of Token Identity Service
    #[serde(default)]
    pub identity_url: Option<Url>,

    /// The URL of Metadata KVS Service
    #[serde(default)]
    pub metadata_url: Option<Url>,

    /// The URL of Locks Service
    #[serde(default)]
    pub locks_url: Option<Url>,
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawDangerZoneConfig {
    /// Allow HTTP in URLs pointing to services
    #[serde(default)]
    #[cfg(feature = "allow_insecure_urls")]
    pub allow_insecure_urls: bool,

    /// Allow invalid or self signed HTTPS certs
    #[serde(default)]
    #[cfg(feature = "allow_invalid_certs")]
    pub allow_invalid_certs: bool,
}

impl RawDangerZoneConfig {
    // Returns `true` if config table is all default values
    pub fn is_empty(&self) -> bool { self == &RawDangerZoneConfig::default() }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deser_empty_config() {
        let cfg_str = r#"
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();
        assert_eq!(cfg, RawConfig::default())
    }

    #[test]
    fn deser_empty_account_config() {
        let cfg_str = r#"
        [account]
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();
        assert_eq!(cfg, RawConfig::default())
    }

    #[test]
    fn deser_empty_seaplane_config() {
        let cfg_str = r#"
        [seaplane]
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();
        assert_eq!(cfg, RawConfig::default())
    }

    #[test]
    fn deser_empty_api_config() {
        let cfg_str = r#"
        [api]
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();
        assert_eq!(cfg, RawConfig::default())
    }

    #[test]
    fn deser_empty_danger_zone_config() {
        let cfg_str = r#"
        [danger-zone]
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();
        assert_eq!(cfg, RawConfig::default())
    }

    #[test]
    fn deser_api_key() {
        let cfg_str = r#"
        [account]
        api-key = "abc123def456"
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();

        assert_eq!(
            cfg,
            RawConfig {
                account: RawAccountConfig { api_key: Some("abc123def456".into()) },
                ..Default::default()
            }
        )
    }

    #[test]
    fn deser_color_key() {
        let cfg_str = r#"
        [seaplane]
        color = "always"
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();

        assert_eq!(
            cfg,
            RawConfig {
                seaplane: RawSeaplaneConfig {
                    color: Some(ColorChoice::Always),
                    default_registry_url: None
                },
                ..Default::default()
            }
        )
    }

    #[test]
    fn deser_default_registry_key() {
        let cfg_str = r#"
        [seaplane]
        default-registry-url = "quay.io/"
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();

        assert_eq!(
            cfg,
            RawConfig {
                seaplane: RawSeaplaneConfig {
                    color: None,
                    default_registry_url: Some("quay.io/".into())
                },
                ..Default::default()
            }
        )
    }

    #[test]
    fn deser_api_urls() {
        let cfg_str = r#"
        [api]
        compute-url = "https://compute.local/"
        identity-url = "https://identity.local/"
        metadata-url = "https://metadata.local/"
        locks-url = "https://locks.local/"
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();

        assert_eq!(
            cfg,
            RawConfig {
                api: RawApiConfig {
                    compute_url: Some("https://compute.local/".parse().unwrap()),
                    identity_url: Some("https://identity.local/".parse().unwrap()),
                    metadata_url: Some("https://metadata.local/".parse().unwrap()),
                    locks_url: Some("https://locks.local/".parse().unwrap()),
                },
                ..Default::default()
            }
        )
    }

    #[cfg(feature = "allow_insecure_urls")]
    #[test]
    fn deser_insecure_urls() {
        let cfg_str = r#"
        [danger-zone]
        allow-insecure-urls = true
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();

        assert_eq!(
            cfg,
            RawConfig {
                danger_zone: RawDangerZoneConfig {
                    allow_insecure_urls: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    }

    #[cfg(feature = "allow_invalid_certs")]
    #[test]
    fn deser_invalid_certs() {
        let cfg_str = r#"
        [danger-zone]
        allow-invalid-certs = true
        "#;

        let cfg: RawConfig = toml::from_str(cfg_str).unwrap();

        assert_eq!(
            cfg,
            RawConfig {
                danger_zone: RawDangerZoneConfig {
                    allow_invalid_certs: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    }
}
