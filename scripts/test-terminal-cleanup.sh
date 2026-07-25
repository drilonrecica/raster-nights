#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0

set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$project_root/target/debug/raster-nights"

for required_command in cargo tmux stty; do
    if ! command -v "$required_command" >/dev/null 2>&1; then
        echo "required command not found: $required_command" >&2
        exit 1
    fi
done

cargo build --manifest-path "$project_root/Cargo.toml" --package raster-nights

qa_directory="$(mktemp -d)"
session_name="raster-nights-cleanup-$$"

cleanup() {
    tmux kill-session -t "$session_name" 2>/dev/null || true
    rm -r -- "$qa_directory"
}
trap cleanup EXIT

run_in_pty() {
    local command="$1"
    local result_file="$2"
    local input="${3:-}"
    local shell_command
    printf -v shell_command \
        'before=$(stty -g); %s; status=$?; after=$(stty -g); if test "$before" = "$after"; then cleanup=ok; else cleanup=changed; fi; printf "status=%%s cleanup=%%s\\n" "$status" "$cleanup" > %q' \
        "$command" "$result_file"

    tmux new-session -d -x 120 -y 40 -s "$session_name" \
        "TERM=xterm-256color bash -c $(printf '%q' "$shell_command")"
    if [[ -n "$input" ]]; then
        tmux send-keys -t "$session_name" -l "$input"
    fi
    for _ in {1..100}; do
        [[ -f "$result_file" ]] && break
        sleep 0.05
    done
    if [[ ! -f "$result_file" ]]; then
        echo "terminal cleanup harness timed out" >&2
        return 1
    fi
    tmux kill-session -t "$session_name" 2>/dev/null || true
}

normal_result="$qa_directory/normal"
run_in_pty "\"$binary\" display-test" "$normal_result" "q"
grep -qx 'status=0 cleanup=ok' "$normal_result"

panic_result="$qa_directory/panic"
run_in_pty "\"$binary\" --test-panic-after-terminal-init" "$panic_result"
grep -Eq '^status=[1-9][0-9]* cleanup=ok$' "$panic_result"

resize_result="$qa_directory/resize"
data_directory="$qa_directory/data"
mkdir "$data_directory"
printf -v resize_command \
    'before=$(stty -g); RASTER_NIGHTS_DATA_DIR=%q %q; status=$?; after=$(stty -g); if test "$before" = "$after"; then cleanup=ok; else cleanup=changed; fi; printf "status=%%s cleanup=%%s\\n" "$status" "$cleanup" > %q' \
    "$data_directory" "$binary" "$resize_result"
tmux new-session -d -x 120 -y 40 -s "$session_name" \
    "TERM=xterm-256color bash -c $(printf '%q' "$resize_command")"
sleep 0.2
tmux resize-window -t "$session_name" -x 80 -y 24
sleep 0.5
small_capture="$(tmux capture-pane -t "$session_name" -p)"
if ! grep -Fq "DRX-90 REQUIRES 100 X 36" <<<"$small_capture"; then
    echo "small-terminal resize screen was not rendered" >&2
    printf '%s\n' "$small_capture" >&2
    exit 1
fi
tmux resize-window -t "$session_name" -x 120 -y 40
sleep 0.5
restored_capture="$(tmux capture-pane -t "$session_name" -p)"
if ! grep -Fq "[ ENTER ] RESUME SESSION" <<<"$restored_capture"; then
    echo "explicit resume screen was not rendered" >&2
    printf '%s\n' "$restored_capture" >&2
    exit 1
fi
tmux send-keys -t "$session_name" Enter
sleep 0.2
launcher_capture="$(tmux capture-pane -t "$session_name" -p)"
if ! grep -Fq "LOCAL SYSTEM NOTICE" <<<"$launcher_capture"; then
    echo "previous screen did not resume after confirmation" >&2
    printf '%s\n' "$launcher_capture" >&2
    exit 1
fi
tmux kill-session -t "$session_name"

echo "terminal cleanup, panic, tmux resize, and explicit resume checks passed"
