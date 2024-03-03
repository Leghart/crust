#!/bin/bash
# TODO!: handle log level with env

BG=""
crust_bin="./target/debug/crust"

function get_fifo_by_pid() {
    pipe="/tmp/tmp_crust_${1}/fifo"
    if [[ -p "$pipe" ]]; then
        echo "$pipe"
    fi
}

function setup_env() {
    pid="$1"
    #TODO: handle case when piddir exists (artifact)
    dir_path="/tmp/tmp_crust_${pid}"
    mkdir "$dir_path"
    mkfifo "${dir_path}/fifo"
    echo "${dir_path}/fifo"
}

function get_bg_process_pid() {
    echo "$(ps aux | grep '[c]rust' | awk '{print $2}' | head -n 1)"
    # echo $(pgrep -f crust)
}

function cleanup() {
    echo "cleanup"
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
                terminate_bg_process "$pid"
                cleanup
                exit 0
            fi
            ;;
        -b | --background)
            BG="-b"
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

pid=$(get_bg_process_pid)
if [[ -z "$pid" ]]; then
    # create background process without command to get a process pid and
    # setup environment for the next calls
    $crust_bin $BG &
    pid="$!"
    fifo="$(setup_env "$pid")"
else
    #  write to existing fifo
    fifo="$(get_fifo_by_pid $pid)"
fi
# echo "CMD: $cmd, fifo: $fifo"
echo $cmd >"$fifo"

if [[ -z "$BG" ]]; then
    echo "INSIDE"
    terminate_bg_process
    cleanup
fi
