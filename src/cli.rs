use crate::environment::Environment;
use crate::settings::Settings;

use miette::{IntoDiagnostic, miette};
use clap::{Parser, Subcommand};
use clap_complete::ArgValueCandidates;
use std::{path::PathBuf, process::exit};
use xshell::Shell;

// TODO repo-run

/// jj-work: a workspace manager for https://jj-vcs.dev
///
/// Homepage: https://github.com/ilyagr/jj-work
///
/// See `jj-work help shell-integration` for details on how to enable shell integration.
#[derive(Parser, Debug)]
#[command(name = "jj-work")]
pub struct Cli {
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
        workspace_name: String,
        /// Passed to `jj workspace add`, see its help for details
        // TODO: add completer. Aside: Seems like `jj workspace add -r` also currently needs one?
        #[arg(long, short, value_name = "REVSETS")]
        revision: Vec<String>,
        // TODO: sparse patterns
    },
    /// Remove a workspace, trying to do it safely
    ///
    /// Will not remove the workspace dir. Will remove any files tracked by `jj`
    /// the `.jj` dir, and any symlinks `jj-work add` would create.
    // TODO: Record symlinks actually created so that we can delete the right ones
    Delete {
        #[arg(add = ArgValueCandidates::new(complete::workspaces))]
        workspace_name: String,
    },
    Path(PathArgs),
    /// Execute a pre-configured shell command inside the given workspace or a
    /// newly created workspace
    //
    // TODO: Set up nice environment variables for the command execution. (Or do
    // we need to?) Info that could be useful:
    //
    // - repo root path (only needed as long as `jj root --repo-root` in not
    //   implemented)
    //
    // - relative path from the original workspace's root to the original
    //   current dir (or should we just cd there? but it might not exist.)
    ExecIn {
        /// Name of the command
        ///
        /// For example, command `NAME` can be defined as a config:
        ///
        /// ```toml
        /// x.jj-work.command.NAME = ["bash", "-c", "echo The workspace is in $(pwd)"]
        /// ```
        #[arg(add = ArgValueCandidates::new(complete::commands))]
        command: String,
        #[command(flatten)]
        args: PathArgs,
    },
    /// List valid workspaces in the vault
    ///
    /// This should be a subset of workspaces that `jj workspace list` would
    /// show, since we only show the workspaces in the vault.
    List,
    /// Print config and environment debug info
    Debug,
    /// The command called by the `jw` shell function.
    ///
    /// A thin alias around other `jj-work` commands, currently `jj-work path`
    #[command(hide = true, name = "jw-command", disable_help_flag = true)]
    JWCommand {
        #[command(flatten)]
        args: PathArgs,
        /// Show help for the `jw` shell function
        #[arg(long, short)]
        help: bool,
    },
    /// Output the README from <https://github.com/ilyagr/jj-work> to the standard output
    ///
    /// If you have `bat` installed, one way to view this highlighted is with
    /// `jj-work docs | bat -l markdown --style=plain`.
    ///
    /// TODO: I don't know of a tool that can render this in a terminal with
    /// internal hyperlinks working.
    Docs,
    /// Install `jj-work`'s shell integration into your shell
    ///
    /// See subcommand description for the exact command to put into your shell config
    ShellIntegration {
        #[command(subcommand)]
        shell: SupportedShells,
    },
}

/// Retrun the path to a workspace or to the repo root
#[derive(Parser, Debug)]
struct PathArgs {
    /// Name of the workspace to switch to
    ///
    /// If not specified, returns the path to the repo root.
    #[arg(add = ArgValueCandidates::new(complete::workspaces))]
    workspace_name: Option<String>,
    /// Use location where the workspace would be, whether or not it's actually
    /// there
    ///
    /// Skip checking whether the target dir exists or whether it contains a
    /// valid workspace for the correct repository
    #[arg(long)]
    allow_missing: bool,
    #[arg(long, short, conflicts_with = "allow_missing")]
    // Could be called `--add-if-missing` to match `jj-work add`, but that would
    // make the short option `-a`. That is easy to confuse with `--allow-missing`
    create_if_missing: bool,
    /// For use with `--create-if-missing`. Passed to `jj workspace add`, see its help for details
    #[arg(long, short, value_name = "REVSETS", requires = "create_if_missing")]
    revision: Vec<String>,
}

fn get_or_create_workspace(
    env: &mut Environment,
    PathArgs {
        workspace_name,
        allow_missing,
        create_if_missing,
        revision,
    }: &PathArgs,
) -> miette::Result<PathBuf> {
    let Some(name) = workspace_name else {
        return Ok(env.repo_root().clone());
    };
    if !allow_missing && !env.is_valid_workspace(name)? {
        if *create_if_missing {
            env.create_workspace(name, revision)?;
        } else {
            return Err(miette::miette!(
                "Workspace '{name}' does not exist or is invalid"
            ));
        }
    }
    Ok(env.path(name))
}

fn path_command(env: &mut Environment, args: &PathArgs) -> miette::Result<()> {
    let path = get_or_create_workspace(env, args)?;
    println!("{}", path.display());
    Ok(())
}

fn exec_in_command(
    env: &mut Environment,
    command_name: &str,
    args: &PathArgs,
) -> miette::Result<()> {
    let mut command = {
        let (program, command_args) = env
            .config
            .command
            .get(command_name)
            .ok_or_else(|| miette!("No command found for name {command_name}."))?
            .split_first()
            .ok_or_else(|| miette!("Command for name {command_name} is empty"))?;
        let mut cmd = std::process::Command::new(program);
        cmd.args(command_args);
        cmd
    };

    let path = get_or_create_workspace(env, args)?;
    command.current_dir(&path);

    let command_status = command.status().into_diagnostic()?;
    if command_status.success() {
        Ok(())
    } else {
        Err(miette! {"Command `{command:?}` failed with {command_status}"})
    }
}

#[derive(Subcommand, Debug)]
enum SupportedShells {
    /// Use as `jj-work shell-integration fish | source`
    Fish,
    /// Use as `source <(jj-work shell-integration bash)`
    Bash,
    /// Use as `source <(jj-work shell-integration zsh)`
    Zsh,
}

impl SupportedShells {
    fn write_script(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self {
            SupportedShells::Fish => {
                writeln!(writer, "{}", include_str!("shell-integration/script.fish"))
            }
            SupportedShells::Bash => {
                writeln!(writer, "source <(COMPLETE=bash jj-work)")?;
                writeln!(writer)?;
                writeln!(writer, "{}", include_str!("shell-integration/script.sh"))
            }
            SupportedShells::Zsh => {
                writeln!(writer, "source <(COMPLETE=zsh jj-work)")?;
                writeln!(writer)?;
                writeln!(writer, "{}", include_str!("shell-integration/script.sh"))
            }
        }
    }
}

mod complete {
    use super::{Environment, setup_env};
    use clap_complete::CompletionCandidate;

    fn with_env<F>(completion_fn: F) -> Vec<CompletionCandidate>
    where
        F: Fn(&Environment) -> Result<Vec<CompletionCandidate>, miette::Report>,
    {
        setup_env()
            .and_then(|env| completion_fn(&env))
            .unwrap_or_else(|_e| {
                // TODO: Log this in some way that isn't visible to the user by default?
                // eprintln!("{}", e);
                Vec::new()
            })
    }

    pub fn workspaces() -> Vec<CompletionCandidate> {
        with_env(|env| {
            Ok(env
                .list_workspaces()
                .unwrap_or_default()
                .into_iter()
                .map(CompletionCandidate::new)
                .collect())
        })
    }

    pub fn commands() -> Vec<CompletionCandidate> {
        with_env(|env| {
            Ok(env
                .config
                .command
                .keys()
                .map(CompletionCandidate::new)
                .collect())
        })
    }
}

fn setup_env() -> miette::Result<Environment> {
    let sh = Shell::new().into_diagnostic()?;
    let config = Settings::new(&sh).into_diagnostic()?;
    let env = Environment::new(&sh, config)?;
    Ok(env)
}

pub fn run(cli: Cli) -> miette::Result<()> {
    // "Early" commands, the commands that need to work outside a repo
    match cli.command {
        Commands::ShellIntegration { shell } => {
            shell.write_script(&mut std::io::stdout()).into_diagnostic()?;
            return Ok(());
        }
        Commands::Docs => {
            // Output the README from the `jj-work` homepage to the standard output
            println!("{}", include_str!("../README.md"));
            return Ok(());
        }
        Commands::JWCommand {
            args: _,
            help: true,
        } => {
            // Help for the `jw` shell function
            // Printed to stderr because `jw` swallows whatever we print to stdout
            eprintln!("jw: call `jj-work path` and cd to the corresponding dir.");
            eprintln!();
            eprintln!("Use `jw space` to jump to a `jj-work` workspace named `space`.");
            eprintln!();
            eprintln!("See `jj-work help path` for additional options.");
            exit(1) // This will cause `jw` to not change the dir
        }
        _ => {}
    }

    let mut env = setup_env()?; // Fails if we are not in a `jj` repo/workspace
    match cli.command {
        Commands::Add {
            workspace_name,
            revision,
        } => env.create_workspace(&workspace_name, &revision)?,
        Commands::Delete { workspace_name } => env.delete_workspace(&workspace_name)?,
        Commands::Path(args) | Commands::JWCommand { args, help: false } => {
            path_command(&mut env, &args)?
        }
        Commands::ExecIn { command, args } => exec_in_command(&mut env, &command, &args)?,
        Commands::List => {
            for workspace_name in env.list_workspaces()? {
                println!("{}", workspace_name);
            }
        }
        Commands::Debug => {
            println!("{:#?}", env);
        }
        Commands::ShellIntegration { .. }
        | Commands::Docs
        | Commands::JWCommand {
            args: _,
            help: true,
        } => {
            panic!("Should be handled earlier, as an early command")
        }
    };
    Ok(())
}
