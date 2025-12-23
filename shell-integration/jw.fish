set -l program_name jw
set -l function_name jwcd

function $function_name -V program_name -d "Change directory to a $program_name workspace"
    set workspace_path ($program_name path $argv) || return 1
    cd $workspace_path
end

complete -e -c $function_name
# TODO: This keeps completing even after `$function_name name <TAB>`
complete -x -c $function_name -a "($program_name list)"
