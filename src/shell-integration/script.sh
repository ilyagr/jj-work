# Shell integration script for Bash and Zsh.
#
# `When `jj-work shell-integration` prints this script, it will also add one of
# the following lines above:
#
# ```bash
# source <(COMPLETE=bash jj-work)
# # Or
# source <(COMPLETE=zsh jj-work)
# ```

function jw {
    local dir
    if dir=$(jj-work jw-command "$@"); then
        test -z "$dir" && return 0
        cd "$dir" || return 1
    else
       return 1
    fi
}

# Todo: completion for `jw`, replicating the `--wraps` in the Fish shell version of the function.
# Could do this specifically for bash/zsh, e.g. for bash see:
# https://stackoverflow.com/questions/55447023/how-do-i-defer-shell-completion-to-another-command-in-bash-and-zsh
