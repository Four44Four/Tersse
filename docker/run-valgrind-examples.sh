#!/usr/bin/env bash
# Run each Cargo example under Valgrind while feeding simulated terminal input via a PTY.
set -euo pipefail

cd /app

readonly VALGRIND=(
    valgrind
    --leak-check=full
    --verbose
    --errors-for-leak-kinds=definite,indirect
    --error-exitcode=1
)

readonly EXAMPLES=(basic_example reflow_example)
readonly TTY_LINES=30
readonly TTY_COLUMNS=100

# Seconds to allow per example (Valgrind slows execution; reflow uses async timers).
declare -A EXAMPLE_TIMEOUT=(
    [basic_example]=60
    [reflow_example]=120
)

pause() {
    sleep "${1:-0.03}"
}

# Writes a mixed keyboard/paste/resize fuzz sequence to stdout (PTY stdin).
emit_fuzz_sequence() {
    local profile="${1:?}"

    pause 0.2

    # Focus tour and navigation
    local i
    for i in $(seq 1 12); do printf '\t'; pause; done
    for i in $(seq 1 6); do printf '\x1b[B'; pause; done
    for i in $(seq 1 6); do printf '\x1b[A'; pause; done
    for i in $(seq 1 4); do printf '\x1b[C'; pause; done
    for i in $(seq 1 4); do printf '\x1b[D'; pause; done
    printf '\x1b[1;2A\x1b[1;2B\x1b[1;3A\x1b[1;3B'
    pause 0.05

    # Text entry, paste burst, and editing
    printf 'fuzz keyboard input for valgrind %s\n' "$profile"
    pause 0.08
    printf '\x1b[200~bracketed paste line one\nline two\twith tab\x1b[201~'
    pause 0.08
    printf 'more typed chars'
    for i in $(seq 1 8); do printf '\x7f'; pause; done
    for i in $(seq 1 4); do printf '\x1b[3~'; pause; done

    # Activate focused controls
    for i in $(seq 1 8); do printf ' '; pause; done
    for i in $(seq 1 4); do printf '\r'; pause; done

    # reflow_example: flash timers (2s / 5s) and extra UI churn
    if [[ "$profile" == "reflow_example" ]]; then
        for i in $(seq 1 6); do printf '\t'; printf ' '; pause 0.4; done
        pause 6
        for i in $(seq 1 4); do printf '\x1b[B'; printf '\r'; pause 0.3; done
    else
        for i in $(seq 1 4); do printf '\t'; printf ' '; pause; done
    fi

    # Terminal resize (SIGWINCH) via stty inside the PTY session is not available here;
    # send a common resize sequence after a short pause.
    pause 0.1
    printf '\x1b[8;25;120t' 2>/dev/null || true
    pause 0.1

    for i in $(seq 1 6); do printf '\t'; pause; done

    # Quit cleanly (avoid Ctrl+C — it delivers SIGINT outside crossterm)
    pause 0.2
    printf '\x1b'
    pause 0.3
}

run_example_under_valgrind() {
    local name="${1:?}"
    local bin="/app/target/debug/${name}"
    local timeout_sec="${EXAMPLE_TIMEOUT[$name]:-90}"
    local log="/tmp/valgrind-${name}.log"

    echo "==> valgrind fuzz: ${name} (timeout ${timeout_sec}s)"

    cargo build --bin "${name}" --quiet

    export TERM=xterm-256color
    export LANG=C.UTF-8
    export LINES="${TTY_LINES}" COLUMNS="${TTY_COLUMNS}"

    local cmd
    cmd="$(printf '%q ' "${VALGRIND[@]}")$(printf '%q' "${bin}")"

    set +e
    timeout "${timeout_sec}" script -qefc "${cmd}" /dev/null \
        > >(tee "${log}") 2>&1 \
        < <(emit_fuzz_sequence "${name}")
    local rc=$?
    set -e

    if [[ "${rc}" -eq 124 ]]; then
        echo "error: ${name} timed out after ${timeout_sec}s (see ${log})" >&2
        return 124
    fi
    if [[ "${rc}" -ne 0 ]]; then
        echo "error: ${name} failed with exit ${rc} (see ${log})" >&2
        return "${rc}"
    fi

    echo "==> ${name}: ok"
}

main() {
    local name
    for name in "${EXAMPLES[@]}"; do
        run_example_under_valgrind "${name}"
    done
    echo "All example Valgrind fuzz runs finished."
}

main "$@"
