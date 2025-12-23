use clap::{Parser, Subcommand};
use jj_work::settings::Settings;
use std::path::PathBuf;
use xshell::{Shell, cmd};

// TODO repo-run

#[derive(Parser, Debug)]
#[command(name = "jj-work")]
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
    /// Retrun the path to a workspace or to the repo root
    Path {
        /// Name of the workspace to switch to
        ///
        /// If not specified, returns the path to the repo root.
        workspace_name: Option<String>,
        #[arg(long)]
        allow_missing: bool,
    },
    /// List valid workspaces in the vault
    ///
    /// This should be a subset of workspaces that `jj workspace list` would
    /// show, since we only show the workspaces in the vault.
    List,
    /// Print config and environment debug info
    Debug,
    ShellIntegration {
        #[command(subcommand)]
        shell: SupportedShells,
    },
}

#[derive(Subcommand, Debug)]
enum SupportedShells {
    Fish,
}

impl SupportedShells {
    fn script(&self) -> &'static str {
        match self {
            SupportedShells::Fish => include_str!("shell-integration/jw.fish"),
        }
    }
}

fn get_repo_root_of_current_dir(sh: &Shell) -> anyhow::Result<PathBuf> {
    // This is normally `repo_root/.jj/repo/config.toml`.
    let repo_config_file: PathBuf = cmd!(sh, "jj config path --repo").read()?.into();
    let repo_root = repo_config_file
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or(anyhow::anyhow!(
            "Couldn't determine repo root from config file path {:?}",
            &repo_config_file
        ))?
        .to_path_buf();
    Ok(repo_root)
}

#[derive(Clone, Debug)]
struct Environment {
    config: Settings,
    _current_dir: PathBuf,
    repo_root: PathBuf,
    _workspace_root: PathBuf,
    // workspaces_dir: PathBuf,
    // repo_config_file: PathBuf,
    _workspace_name: Option<String>,
}

impl Environment {
    fn new(sh: &Shell, config: Settings) -> anyhow::Result<Self> {
        // TODO: Non-UTF-8?
        let workspace_root: PathBuf = cmd!(sh, "jj workspace root").read()?.into();
        let repo_root = get_repo_root_of_current_dir(sh)?;

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
            config,
            _current_dir: sh.current_dir(),
            repo_root,
            _workspace_root: workspace_root,
            _workspace_name: workspace_name,
        })
    }

    fn vault_dir(&self) -> PathBuf {
        // TODO: Git commands still work?
        self.repo_root.join(&self.config.vault_dir)
    }

    fn repo_shell(&self) -> anyhow::Result<Shell> {
        let sh = Shell::new()?;
        sh.change_dir(&self.repo_root);
        Ok(sh)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.vault_dir().join(name)
    }

    /// The dir returned may not exist!
    fn workspace_shell(&self, name: &str) -> anyhow::Result<Shell> {
        let sh = Shell::new()?;
        sh.change_dir(self.path(name));
        Ok(sh)
    }

    fn create_workspace(&mut self, name: &str) -> anyhow::Result<()> {
        let sh = self.repo_shell()?;
        sh.create_dir(self.vault_dir())?;
        let workspace_path = self.vault_dir().join(name);
        // TODO: Test
        cmd!(sh, "jj workspace add {workspace_path}").run()?;
        // TODO: Check gitignores?
        for need_symlink in self.config.paths_to_symlink.iter() {
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

    fn is_valid_workspace(&self, name: &str) -> anyhow::Result<bool> {
        // TODO: Create `jj workspace name`, then we can compare `jj workspace name` with the dir name and error if they are different.
        // (We probably won't support `jj workspace add --name`)
        // TODO: Create `jj workspace repo`, or adjust template

        // TODO: test
        let sh = self.workspace_shell(name)?;
        if !sh.path_exists(".jj") {
            return Ok(false);
        }
        let workspace_root = get_repo_root_of_current_dir(&sh)?;
        Ok(workspace_root == self.repo_root)
    }

    fn list_workspaces(&self) -> anyhow::Result<Vec<String>> {
        let sh = self.repo_shell()?;
        Ok(sh
            .read_dir(self.vault_dir())?
            .into_iter()
            .filter_map(|entry| {
                entry
                    .file_name()
                    .and_then(|os_str| os_str.to_str())
                    .map(|s| s.to_string())
            })
            .filter(|name| self.is_valid_workspace(name).unwrap_or(false))
            .collect())
    }

    // TODO: Delete workspace, really belongs to `jj`. Set sparse pattern to `!*`, then figure out ignore files.
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Commands that need to work outside a repo
    #[expect(clippy::single_match)]
    match cli.command {
        Commands::ShellIntegration { shell } => {
            println!("{}", shell.script());
            return Ok(());
        }
        _ => {}
    }

    let sh = Shell::new().unwrap();
    let config = Settings::new(&sh)?;
    let mut env = Environment::new(&sh, config.clone()).unwrap();
    match cli.command {
        Commands::Add { workspace_name } => env.create_workspace(&workspace_name)?,
        Commands::Path {
            workspace_name: None,
            allow_missing: _,
        } => {
            println!("{}", env.repo_root.display())
        }
        Commands::Path {
            workspace_name: Some(name),
            allow_missing,
        } => {
            if !allow_missing && !env.is_valid_workspace(&name)? {
                return Err(anyhow::anyhow!(
                    "Workspace '{name}' does not exist or is invalid"
                ));
            }
            let path = env.path(&name);
            println!("{}", path.display());
        }
        Commands::List => {
            for workspace_name in env.list_workspaces()? {
                println!("{}", workspace_name);
            }
        }
        Commands::ShellIntegration { .. } => panic!("Should be handled earlier"),
        Commands::Debug => {
            println!("{:#?}", env);
        }
    };
    Ok(())
}
