# `jj-work`, a workspace manager for `jj`

[GitHub homepage](https://github.com/ilyagr/jj-work)

`jj-work` is a workspace manager for [`jj`](https://jj-vcs.dev) that is meant to
help with using [`jj` workspaces][workspaces]. `jj-work` wraps the regular [`jj
workspace` commands][jj-workspace cli].

[workspaces]: https://docs.jj-vcs.dev/latest/working-copy/#workspaces
[jj-workspace cli]: https://docs.jj-vcs.dev/latest/cli-reference/#jj-workspace

## Warning: experimental code

This is currently an unpolished and unfinished proof-of-concept. I'd like to find out what
features in `jj-work` or `jj` would improve these kinds of workflows. In it's
current state, `jj-work` is likely to be most appropriate for people who are
also interested in developing `jj` workspace workflows, and are willing to work
through any surprises that might come up.

There is no CI, no version number, and no promise of stability. Branches other
than `trunk` are likely to be force-pushed to, and `trunk` is not safe from
force-pushes either. If and as the project eventually matures, these things will
change (starting with the "no CI", which is a little embarassing).

## Installation and setup

Windows is not currently supported. WSL should be fine.

For now, this tool should be installed by compiling from a clone of this repo.
Rust is required. For example:

```bash
jj git clone -b trunk https://github.com/ilyagr/jj-work
cd jj-work
cargo install --path $(jj root)
```

Then, you should set up the shell integration for your shell and consider
whether you'd like to [configure `jj-work` to create symlinks in new
workspaces](#creating-symlinks) for some of the repos you will use `jj-work` on. 

### Shell integration: Fish shell

Simple option: Put `jj-work shell-integration fish | source` anywhere in
your config.

Suggested option:

```fish
mkdir -p ~/.config/fish/conf.d
begin
    echo 'status is-interactive &&'
    echo 'type -q jj-work &&'
    echo 'jj-work shell-integration fish | source'
end > ~/.config/fish/conf.d/jj-work.fish
```

Then, restart `fish`, e.g. by running `exec fish`.

### Shell integration: Bash and Zsh

Put one of these in your config:

```bash
source <(jj-work shell-integration bash)
```

or

```bash
source <(jj-work shell-integration zsh)
```

Then restart your shell.

This defines the `jw` function and sets up the `jj-work` command-line
completion, but there currently is no command-line completion for `jw`. Bash and
Zsh are currently less tested than Fish, improvements are welcome.

### Other shells

Proper integration is TODO, see [suggestions on implementing shell support
below](#notes-on-implementing-support-for-more-shells).

Workaround: you can use the `jj-work add` command together with commands such as
`cd $(jj-work path space)`.

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

### Future Plans

- Better ways to handle the issue of stale workspaces. For example, nudge users
  towards creating workspaces on commits that are unlikely to interfere with
  each other.
- Auto-deduplicating workspace name (e.g. add date to them)
- `../{repo_name}-work` vault path support.
  https://crates.io/crates/tinytemplate or
  https://github.com/mitsuhiko/minijinja?
- allow run templated scripts instead of simple `cd`-ing to a workspace. Will reduce the need for shell integration
- tmux integration
- Put `.gitignore` in vault dir?
- Windows (first, make it compile without symlink support, and then implement
  symlink support)
- Better integration with Git worktrees: [jj issue
  #8025](https://github.com/jj-vcs/jj/issues/8052), we could also create a
  `.git` for workspaces inside subdirs colocated repos to stop Git from
  operating on the main repo.
- More shells (see just below)

#### Notes on implementing support for more shells

Help wanted! There are a few tasks to be done for each shell.

Most importantly, the end result should be tested by a user of the relevant
shell.

- Set up `jj-work shell-integration <shell>` with Clap's completion. Document
  the way to use the command to configure the shell in this file and in the Clap
  help (docstring for the enum option you will need to create for the new shell).

- Create a `jw` shell function for each shell that runs `jj-work jw-command` and, if that succeeds and prints something to stdout, changes the dir to the path it returned.

  For guidance, see [the fish version](src/shell-integration/script.fish), [shell
  scripts that define `br` functions inside Rust files for
  `broot`](https://github.com/Canop/broot/tree/main/src/shell_install) and
  various `lfcd` examples in <https://github.com/gokcehan/lf/tree/master/etc>
  (though they don't illustrate checking the exit code)

- Create completion for the `jw` shell function. It should be the same as the
  completion Clap set up for `jj-work jw-command`.

  This is not as important, and might be harder, but is quite helpful.

  Setting up the `jw` completion could be done for each shell individually (in
  the case of Fish, for example, it is currently done merely by adding a
  `--wraps` command to the Fish function definition), or see the [approach `uv`
  takes for the `uvx` command-line completions][uv approach] for how it might be
  possible via a more advanced use of Clap (which would need testing).

[uv approach]: https://github.com/astral-sh/uv/blob/4269f889bb57b3dd80fd3158c3fac7921592dd5e/crates/uv/src/lib.rs#L1289-L1311

## User Guide (Draft)

A `jj-work` workspace is just a normal `jj` workspace that is located in a child
directory of a special "`jj-work` vault" directory, and has the same name as its
directory. So, the output of `jj-work list` will contain a subset of the
workspaces `jj workspace list` shows.

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

### Dealing with stale workspaces

TODO

### No main repo

TODO

<https://discord.com/channels/968932220549103686/968932220549103689/1325266777512480790>

Probably `jj sparse set` is better. (Link to gitignore-changing issues?)

Where to put the vault?


## Development

My own development usually happens on the `dev` branch, but if you are
considering creating a PR, the default `trunk` branch is probably best as a
base. I often force-push `dev`.

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

