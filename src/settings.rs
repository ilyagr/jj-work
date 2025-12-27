use std::{collections::HashMap, path::PathBuf};

use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Deserializer};
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

// TODO: Move some of the doc comment below here and to the README? TODO: Have
// separate TOML keys for the vector and the mapping that are combined together.
fn deserialize_paths<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PathOrPaths {
        Single(PathBuf),
        Multiple(Vec<PathBuf>),
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PathsFormat {
        Vector(Vec<PathBuf>),
        Mapping(HashMap<String, PathOrPaths>),
    }

    let format = PathsFormat::deserialize(deserializer)?;

    match format {
        PathsFormat::Vector(paths) => Ok(paths),
        PathsFormat::Mapping(map) => Ok(map
            .into_values()
            .flat_map(|value| match value {
                PathOrPaths::Single(path) => vec![path],
                PathOrPaths::Multiple(paths) => paths,
            })
            .collect()),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// Relative to the repo root, unlike the CLI option
    // TODO: substitute {repo_name}
    pub vault_dir: PathBuf,
    /// Relative to the repo root
    ///
    /// These can be stored in a few different formats in TOML:
    ///
    /// - Simple array: `paths_to_symlink = ["path1", "path2"]
    /// - A table with string or array values. This can help combine values from
    ///   user-level and repo-level configs, since `jj` currently doesn't
    ///   concatenate arrays when combining such values.
    ///
    /// For example, one can specify the list of paths as follows:
    ///
    /// ```toml
    /// [x.jj-work]  # In user-level config
    /// paths_to_symlink.user = ["path1", "path2"]
    ///
    /// [x.jj-work]  # In repo-level config
    /// paths_to_symlink.repo = ["path3", "path4"]
    /// paths_to_symlink.named_path = "path5"
    /// ```
    ///
    /// The key values of this table matter only while `jj` generates the table,
    /// and are ignored by `jj-work`. If you use the same key in user-level and
    /// repo-level config, the values will not be appended; only the repo-level
    /// value will be retained.
    //
    // TODO: Link to docs for the config crate
    #[serde(deserialize_with = "deserialize_paths")]
    pub paths_to_symlink: Vec<PathBuf>,
    pub command: HashMap<String, Vec<String>>,
}

impl Settings {
    pub fn new(sh: &Shell) -> Result<Self, SettingsError> {
        // TODO: test As discussed in https://github.com/jj-vcs/jj/pull/8379,
        // this is one way to get a config key as valid TOML.
        let template = r#"name ++ "=" ++ value ++ "\n""#;
        let config_form_jj_str = cmd!(
            sh,
            "jj config list --ignore-working-copy --include-defaults -T {template} --color=never x.jj-work"
        )
        .ignore_stderr()
        .read()
        .unwrap_or_default();
        let config_from_jj: toml::Value = toml::from_str::<toml::Table>(&config_form_jj_str)?
            .get("x")
            .and_then(|x| x.get("jj-work"))
            .cloned()
            .unwrap_or(toml::Value::Table(Default::default()));
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from_str(
                r#"
                vault_dir = ".jj/jj-work"  # "../{repo_name}-ws
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
