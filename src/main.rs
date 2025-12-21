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
    // #[arg(long, global = true)]
    // vault: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new workspace
    Add {
        /// Name of the workspace to add
        workspace_name: String,
        // TODO: Revision, sparse patterns
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

    fn vault_dir(&self) -> PathBuf {
        // Or _workspaces/ .jj/workspaces-jw/, or ../{repo_name}_workspaces
        // TODO: Git commands still work?
        self.repo_root.join(".jj/jw-workspaces")
    }

    fn repo_shell(&self) -> anyhow::Result<Shell> {
        let sh = Shell::new()?;
        sh.change_dir(&self.repo_root);
        Ok(sh)
    }

    fn create_workspace(&mut self, name: String) -> anyhow::Result<()> {
        let sh = self.repo_shell()?;
        sh.create_dir(self.vault_dir())?;
        let workspace_path = self.vault_dir().join(name);
        // TODO: Test
        cmd!(sh, "jj workspace add {workspace_path}").run()?;
        // TODO: Check gitignores?
        for need_symlink in ["target", "_ilyagrignore", "node_modules"] {
            // TODO: Windows
            std::os::unix::fs::symlink(
                self.repo_root.join(need_symlink),
                workspace_path.join(need_symlink),
            )?;
        }
        // TODO: update_stale options?
        // TODO: `echo "gitdir: /dev/null" > .git`` if colocated, or is this jj's job?
        // See also GIT_CEILING_DIRECTIORIES, https://stackoverflow.com/questions/27177248/how-can-i-make-git-work-only-on-the-current-directory
        // jj discussion on making co-located workspaces work with git
        Ok(())
    }

    // TODO: Delete workspace, really belongs to `jj`. Set sparse pattern to `!*`, then figure out ignore files.
}

fn main() -> anyhow::Result<()> {
    let sh = Shell::new().unwrap();
    let mut env = Environment::new(&sh).unwrap();
    eprintln!("{:#?}", env);
    match Cli::try_parse() {
        Ok(cli) => match cli.command {
            Commands::Add { workspace_name } => env.create_workspace(workspace_name)?,
            //Commands::Switch { workspace_name } => todo!(),
            _ => eprintln!("Unimplemented: {:#?}", cli),
        },
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
    };
    Ok(())
}
