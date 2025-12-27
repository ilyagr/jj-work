set -l program_name jj-work
set -l function_name jw
set -l abbr_name jx

complete -e $program_name
# TODO: Move this inside Rust code or a separate shell file and have a dedicated
# file for defining `jw`?
COMPLETE=fish $program_name | source

complete -e $function_name  # Completion is created by `--wraps` option below
function $function_name --wraps "$program_name jw-command" -V program_name \
                        -d "Change directory to a $program_name workspace"
    set workspace_path ($program_name jw-command $argv) || return 1
    test -z "$workspace_path" && return 0  # To avoid jumping to user's home unexpectedly, and to allow explicit decision to stay in place
    cd $workspace_path
end

abbr $abbr_name $program_name exec-in
