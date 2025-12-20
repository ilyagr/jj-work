use duct::cmd;
use std::fs;
use std::path::PathBuf;

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
        // TODO: Non-UTF-8?
        let workspace_root: PathBuf = cmd!("jj", "workspace", "root").read()?.trim().into();
        let dot_jj = workspace_root.join(".jj");

        let (repo_root, workspace_name) = if fs::metadata(&dot_jj)?.is_dir() {
            (workspace_root.clone(), None)
        } else {
            (
                fs::read_to_string(&dot_jj)?.trim().into(),
                Some(
                    workspace_root
                        .file_name()
                        .and_then(|os_str| os_str.to_str())
                        .ok_or(anyhow::anyhow!(
                            "Couldn't determine workspace name from path {workspace_root:?}"
                        ))?
                        .to_string(),
                ),
            )
        };

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
