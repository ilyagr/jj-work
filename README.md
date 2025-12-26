# `jj-work`, a workspace manager for `jj`

[Homepage](https://github.com/ilyagr/jj-work)

`jj-work` is a workspace manager for [`jj`](https://jj-vcs.dev) that is meant to
help with using [`jj` workspaces][workspaces]. `jj-work` wraps the regular [`jj
workspace` commands][jj-workspace cli].

This is currently an unpolished proof-of-concept.

[workspaces]: https://docs.jj-vcs.dev/latest/working-copy/#workspaces
[jj-workspace cli]: https://docs.jj-vcs.dev/latest/cli-reference/#jj-workspace

## Features

If [shell integration](#installation-and-shell-integration) is installed,
`jj-work` can be used via the `jw` (mnemonic: "**j**ump to **w**orkspace") shell
function:

- `jw -c new-space` creates a workspace named `new-space` and jumps to it

    - You can configure `jj-work` to [create symlinks for gitignored
      files](#creating-symlinks) from the new workspace to the main repos. For
      example, you can link `new-workspace/node_modules` to the `node_modules`
      from the main repo automatically.

- `jw space` jumps (`cd`-s) to a previously created workspace `space`

- `jw <TAB>` uses command-line completion to see what workspaces are available
  (currently only supported in the Fish shell)

- `jw` by itself jumps to the main repo

For other features, or if shell integration is not installed, use the `jj-work`
binary. Run `jj-work help` for a list of commands.

For example, `jj-work delete space` improves on `jj workspace forget` by
deleting all the tracked files from the workspace and the `.jj` dir. (TODO:
  Consider adding a version of this to `jj` proper)

### Features TODO

- More shells
- tmux integration
- Auto-deduplicating workspace name (e.g. add date to them)
- `../{repo_name}-work` vault path support.
  https://crates.io/crates/tinytemplate or
  https://github.com/mitsuhiko/minijinja?
- Put `.gitignore` in vault dir?
- Windows (without symlinks, and then symlink support)
- Better integration with Git worktrees (TODO: Link to jj issues, perhaps discuss `.git` creation)

## User Guide

A `jj-work` workspace is just a normal `jj` workspace that is located in a child
directory of a special "`jj-work` vault" directory, and has the same name as its
directory. So, the output of `jj-work list` will contain a subset of the
workspaces `jj workspace list` shows.

### Installation and shell integration

### Configuration

`jj-work` reads config from the `x.jj-work` table in the jj's config. 

You can see the current config with `jj-work debug`.

TODO: more details

#### Creating symlinks

Example: To configure a repo where a user wants to share the Rust/Cargo binary
cache in the `target/` dir and their VS Code workspace config, they can do:

```bash
jj config set --repo 'x.jj-work.paths_to_symlink = ["target", ".vscode/settings.json"]'
```

## Tips and tricks

### No main repo

TODO

<https://discord.com/channels/968932220549103686/968932220549103689/1325266777512480790>

Probably `jj sparse set` is better. (Link to gitignore-changing issues?)

Where to put the vault?


## Development

Shell script-quality in places

Need tests to replace `xshell`

## Related tools

### Git workspace managers

I am aware of a few workspace managers for Git. (They may also be called
"worktree managers", since Git's analogue to jj's workspaces is a worktree)

- [workmux](https://github.com/raine/workmux)

- [wtp](https://github.com/satococoa/wtp)

The main difference between them and `jj-work` is that `jj-work` can do less,
because it can rely on `jj` to do more. For example, Git workspace managers
usually create Git worktrees together with a new Git branch, and expect the
branch to continue to correspond to the workspace. OTOH, `jj-work` never needs
to create jj bookmarks.

Other than that, the Git workspace managers are more polished and have
interesting ideas `jj-work` either already borrowed (e.g. the symlink creation)
or may want to borrow in the future (e.g. tmux integration of `workmux`) in some
form.

