#!/bin/bash

BG_FLAG=""
crust_bin="cargo run -- " #TODO: to change before release version

function get_fifo_by_pid() {
    pipe="/tmp/tmp_crust_${1}/fifo"
    if [[ -p "$pipe" ]]; then
        echo "$pipe"
    fi
}

function setup_env() {
    dir_path="/tmp/tmp_crust_${1}"
    mkdir "$dir_path"
    mkfifo "${dir_path}/fifo"
    echo "${dir_path}/fifo"
}

function get_bg_process_pid() {
    echo "$(ps aux | grep '[c]rust' | awk '{print $2}' | head -n 1)"
}

function cleanup() {
    dir_path="/tmp/tmp_crust_${1}"
    rm -rf "$dir_path"
}

function terminate_bg_process() {
    kill -15 "$1"
}

function show_help() {
    echo "Usage: $0 [options]... [command]..."
    echo "Command:"
    echo " exec       Execute command on requested machine"
    echo " scp        Copy data between requested machines"
    echo "Options:"
    echo "  -h        Show help."
    echo "  -b        Run in background."
    echo "  -e        Exit background process (if exists)."
    exit 0
}

for i in "$@"; do
    case $i in
        -h | --help)
            show_help
            ;;
        -e | --exit)
            pid=$(get_bg_process_pid)
            if [[ -z "$pid" ]]; then
                echo >&2 "No active crust process to termination"
                exit 1
            else
                cleanup "$pid"
                terminate_bg_process "$pid"
                exit 0
            fi
            ;;
        -b | --background)
            BG_FLAG=true
            export CRUST_BG_MODE=true
            shift 1
            ;;
        --) # every argument after that flag will be stored in $cmd
            shift
            break
            ;;
        *)
            echo "Unexpected option: $1"
            shift 1
            ;;
    esac
done

cmd="$*"
export CRUST_SHELL_INVOKE=true

if [[ -z "$BG_FLAG" ]]; then
    $crust_bin $cmd
    exit "$?"
else
    pid=$(get_bg_process_pid)
    if [[ -z "$pid" ]]; then
        # create background process without command to get a process pid and
        # setup environment for the next calls
        $crust_bin -b &
        pid="$!"
        fifo="$(setup_env "$pid")"
    else
        #  pass command to existing fifo
        fifo="$(get_fifo_by_pid $pid)"
    fi
    echo $cmd >"$fifo"
fi

if [[ -z "$BG_FLAG" ]]; then
    cleanup "$pid"
    terminate_bg_process "$pid"
fi
