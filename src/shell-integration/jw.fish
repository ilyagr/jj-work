set -l program_name jj-work
set -l function_name jw

COMPLETE=fish $program_name | source

function $function_name -V program_name -d "Change directory to a $program_name workspace"
    set workspace_path ($program_name path $argv) || return 1
    test -z "$workspace_path" && return 0  # For safety, and to allow explicit decision to stay in place
    cd $workspace_path
end

complete -e -c $function_name
# TODO: This keeps completing even after `$function_name name <TAB>`
complete -x -c $function_name -a "($program_name list 2>/dev/null)"
