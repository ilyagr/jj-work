use std::{
    io::{Write, stderr},
    path::PathBuf,
};
use xshell::{Shell, cmd};

use crate::settings::Settings;

fn get_repo_root_of_current_dir(sh: &Shell) -> anyhow::Result<PathBuf> {
    // This is normally `repo_root/.jj/repo/config.toml`.
    let repo_config_file: PathBuf = cmd!(sh, "jj config path --repo --ignore-working-copy")
        .read()?
        .into();
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
pub struct Environment {
    config: Settings,
    pub repo_root: PathBuf,
    _current_dir: PathBuf,
    _current_workspace_root: PathBuf,
    // workspaces_dir: PathBuf,
    // repo_config_file: PathBuf,
    _current_workspace_name: Option<String>,
}

impl Environment {
    pub fn new(sh: &Shell, config: Settings) -> anyhow::Result<Self> {
        // TODO: Non-UTF-8?
        let workspace_root: PathBuf = cmd!(sh, "jj workspace root --ignore-working-copy")
            // Would be nice to capture the stderr and print it as part of
            // error. However, we shouldn't let `jj workspace root` print it
            // since this call happens during command-line completion, which
            // needs to suppress all error output.
            .ignore_stderr()
            .read()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Current directory must be in a jj repo/workspace, but we failed to find it: {e}"
                )
            })?
            .into();
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
            _current_workspace_root: workspace_root,
            _current_workspace_name: workspace_name,
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

    pub fn path(&self, name: &str) -> PathBuf {
        self.vault_dir().join(name)
    }

    /// The dir returned may not exist!
    pub fn workspace_shell(&self, name: &str) -> anyhow::Result<Shell> {
        let sh = Shell::new()?;
        sh.change_dir(self.path(name));
        Ok(sh)
    }

    pub fn create_workspace(&mut self, name: &str, revisions: &[String]) -> anyhow::Result<()> {
        let sh = self.repo_shell()?;
        sh.create_dir(self.vault_dir())?;
        let workspace_path = self.vault_dir().join(name);

        match revisions {
            [] => {
                cmd!(sh, "jj workspace add {workspace_path}").run()?;
            }
            [revision] => {
                cmd!(sh, "jj workspace add -r {revision} {workspace_path}").run()?;
            }
            [first, second] => {
                cmd!(
                    sh,
                    "jj workspace add -r {first} -r {second} {workspace_path}"
                )
                .run()?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Currently at most two `-r` arguments are supported when creating a workspace"
                ));
            }
        }
        // TODO: Test
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

    pub fn delete_workspace(&mut self, name: &str) -> anyhow::Result<()> {
        if !self.is_valid_workspace(name)? {
            return Err(anyhow::anyhow!(
                "Workspace '{name}' does not exist or is invalid",
            ));
        }
        let sh = self.workspace_shell(name)?;

        let output = cmd!(sh, "jj workspace update-stale").output()?;
        // We want to suppress the output unless there is an error
        if !output.status.success() {
            let _ = stderr().write_all(&output.stdout); // Likely empty
            let _ = stderr().write_all(&output.stderr);
            // TODO: Do we want: let _ = stderr().write_all(b"\n");
            return Err(anyhow::anyhow!(
                "`jj workspace update-stale` failed with exit code {:?}",
                output.status.code()
            ));
        }

        cmd!(sh, "jj sparse set --clear").run()?;
        cmd!(sh, "jj workspace forget {name}").run()?;
        if sh.path_exists(".jj") {
            // eprintln!("Would remove: {:?}", sh.read_dir(".jj")?);
            sh.remove_path(".jj")?;
        }

        for symlink_to_remove in self.config.paths_to_symlink.iter() {
            let full_path = sh.current_dir().join(symlink_to_remove);
            match std::fs::read_link(&full_path) {
                Ok(target) if target == self.repo_root.join(symlink_to_remove) => {
                    // https://github.com/matklad/xshell/issues/106:
                    // `sh.remove_path(symlink_to_remove)?` doesn't work on
                    // symlinks that don't point to existing files
                    std::fs::remove_file(full_path)?;
                }
                _ => {
                    // TODO: Log error
                }
            }
        }

        Ok(())
    }

    pub fn is_valid_workspace(&self, name: &str) -> anyhow::Result<bool> {
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

    pub fn list_workspaces(&self) -> anyhow::Result<Vec<String>> {
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
}
