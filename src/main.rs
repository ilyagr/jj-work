use duct::cmd;
use std::path::PathBuf;

// TODO repo-run

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
    fn new_from_cwd() -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir()?;
        // TODO: Non-UTF-8? TODO: Mention failure to run jj in error messages.
        // TODO: print stderr on error? Or does this already happen?
        let workspace_root: PathBuf = cmd!("jj", "workspace", "root").read()?.trim().into();

        // This is normally `repo_root/.jj/repo/config.toml`.
        let repo_config_file: PathBuf =
            cmd!("jj", "config", "path", "--repo").read()?.trim().into();
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
            current_dir,
            repo_root,
            workspace_root,
            workspace_name,
        })
    }
}

fn main() {
    let env = Environment::new_from_cwd().unwrap();
    eprintln!("{:#?}", env);
}
