set -l program_name jj-work
set -l function_name jw

# TODO: Move this inside Rust code or a separate shell file to distinguish `jw.fish`?
COMPLETE=fish $program_name | source

function $function_name --wraps "$program_name jw-command" -V program_name \
                        -d "Change directory to a $program_name workspace"
    set workspace_path ($program_name jw-command $argv) || return 1
    test -z "$workspace_path" && return 0  # To avoid jumping to user's home unexpectedly, and to allow explicit decision to stay in place
    cd $workspace_path
end

# Todo: cross-shell completion for `jw` to replicate the `--wraps` above. (Also,
# `jw` needs to be written for other shells, but that's easier). E.g. follow
# https://github.com/astral-sh/uv/blob/4269f889bb57b3dd80fd3158c3fac7921592dd5e/crates/uv/src/lib.rs#L1289-L1311

# Or bash: https://stackoverflow.com/questions/55447023/how-do-i-defer-shell-completion-to-another-command-in-bash-and-zsh
