use clap::{Parser, Subcommand};
use std::path::PathBuf;
use xshell::{Shell, cmd};

// TODO repo-run

#[derive(Parser, Debug)]
#[command(name = "jw")]
#[command(about = "Jujutsu workspace manager", long_about = None)]
struct Cli {
    /// Optional path to vault where new workspaces are created and looked for
    ///
    /// There can be multiple vaults anywhere in the filesystem.
    // TODO: If not specified... When and whether gitignore is created in it.
    // TODO: One repo per vault? Non-workspace dirs in vault? (Maybe OK if they don't have .jj)
    // TODO: Probably a list of repo-relative vaults in config, but CLI option is relative to CWD.
    #[arg(long, global = true)]
    vault: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new workspace
    Add {
        /// Name of the workspace to add
        workspace_name: String,
    },
    /// Switch to a workspace
    Switch {
        /// Name of the workspace to switch to
        workspace_name: String,
    },
}

#[derive(Clone, Debug)]
struct Environment {
    current_dir: PathBuf,
    repo_root: PathBuf,
    workspace_root: PathBuf,
    // workspaces_dir: PathBuf,
    // repo_config_file: PathBuf,
    workspace_name: Option<String>,
}

impl Environment {
    fn new(sh: &Shell) -> anyhow::Result<Self> {
        // TODO: Non-UTF-8?
        let workspace_root: PathBuf = cmd!(sh, "jj workspace root").read()?.into();

        // This is normally `repo_root/.jj/repo/config.toml`.
        let repo_config_file: PathBuf = cmd!(sh, "jj config path --repo").read()?.into();
        // TODO: Maybe better to put a file with path to repo in the workspaces dir? Then, jjw could work in that dir as well.
        let repo_root = repo_config_file
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .ok_or(anyhow::anyhow!(
                "Couldn't determine repo root from config file path {:?}",
                &repo_config_file
            ))?
            .to_path_buf();

        let workspace_name = (repo_root != workspace_root)
            .then(|| {
                workspace_root
                    .file_name()
                    .and_then(|os_str| os_str.to_str())
                    .ok_or(anyhow::anyhow!(
                        "Couldn't determine workspace name from path {workspace_root:?}"
                    ))
                    .map(|s| s.to_string())
            })
            .transpose()?;

        Ok(Self {
            current_dir: sh.current_dir(),
            repo_root,
            workspace_root,
            workspace_name,
        })
    }
}

fn main() {
    match Cli::try_parse() {
        Ok(cli) => eprintln!("{:#?}", cli),
        Err(e)
            if matches!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelp
                    | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ) =>
        {
            e.print().unwrap();
            std::process::exit(0);
        }
        Err(e) => {
            let mut descr = e.kind().to_string();
            if descr.is_empty() {
                descr = format!("{:?}", e.kind());
            }
            eprintln!("clap: {}", descr)
        }
    }
    let sh = Shell::new().unwrap();
    let env = Environment::new(&sh).unwrap();
    eprintln!("{:#?}", env);
}
