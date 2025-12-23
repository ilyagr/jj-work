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

const JJ_CONFIG_KEY: &str = "x.jw";

impl Settings {
    pub fn new(sh: &Shell) -> Result<Self, SettingsError> {
        // TODO: test As discussed in https://github.com/jj-vcs/jj/pull/8379,
        // this is one way to get a config key as valid TOML.
        let template = r#"name ++ "=" ++ value ++ "\n""#;
        let jj_config = cmd!(
            sh,
            "jj config list --include-defaults -T {template} --color=never {JJ_CONFIG_KEY}"
        )
        .ignore_stderr()
        .read()
        .unwrap_or_default();
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
            .add_source(File::from_str(&jj_config, FileFormat::Toml))
            .build()?;
        Ok(s.try_deserialize()?)
    }
}
