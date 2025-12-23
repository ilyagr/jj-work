use std::path::PathBuf;

use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;
use thiserror::Error;
use xshell::{Shell, cmd};

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("Shell command error: {0}")]
    Shell(#[from] xshell::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// Relative to the repo root, unlike the CLI option
    // TODO: substitute {repo_name}
    pub vault_dir: PathBuf,
    /// Relative to the repo root
    pub paths_to_symlink: Vec<PathBuf>,
}

impl Settings {
    pub fn new(sh: &Shell) -> Result<Self, SettingsError> {
        // TODO: test As discussed in https://github.com/jj-vcs/jj/pull/8379,
        // this is one way to get a config key as valid TOML.
        let template = r#"name ++ "=" ++ value ++ "\n""#;
        let config_form_jj_str = cmd!(
            sh,
            "jj config list --include-defaults -T {template} --color=never x.jw"
        )
        .ignore_stderr()
        .read()
        .unwrap_or_default();
        let config_from_jj: toml::Value = toml::from_str::<toml::Table>(&config_form_jj_str)?
            .get("x")
            .and_then(|x| x.get("jw"))
            .cloned()
            .unwrap_or(toml::Value::Table(Default::default()));
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from_str(
                r#"
                vault_dir = ".jj/jw-vault"  # "../{repo_name}-ws
                paths_to_symlink = []
                # test-unknown = "fred"
            "#,
                FileFormat::Toml,
            ))
            .add_source(config::Config::try_from(&config_from_jj)?)
            .build()?;
        Ok(s.try_deserialize()?)
    }
}
