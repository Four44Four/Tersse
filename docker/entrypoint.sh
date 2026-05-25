#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF' >&2
Usage: docker run [options] tersse-valgrind [MODE] [MODE_ARGS...]

Modes:
  tests      Run ./tests integration tests under Valgrind (default)
  examples   Run example binaries under Valgrind with keyboard fuzzing

Examples:
  docker run --rm tersse-valgrind
  docker run --rm tersse-valgrind tests -- --nocapture
  docker run --rm tersse-valgrind tests pure_button
  docker run --rm tersse-valgrind examples
EOF
}

mode="${1:-tests}"
case "${mode}" in
    -h | --help | help)
        usage
        exit 0
        ;;
    tests | examples)
        shift
        ;;
    *)
        echo "error: unknown mode '${mode}'" >&2
        usage
        exit 2
        ;;
esac

case "${mode}" in
    tests) exec /app/docker/run-valgrind-tests.sh "$@" ;;
    examples) exec /app/docker/run-valgrind-examples.sh "$@" ;;
esac
