#!/usr/bin/env bash
set -euo pipefail

cd /app

host="$(rustc -vV | awk '/^host: / { print $2 }')"
runner_var="CARGO_TARGET_${host^^}"
runner_var="${runner_var//-/_}_RUNNER"
export "${runner_var}=valgrind --leak-check=full --verbose --errors-for-leak-kinds=definite,indirect --error-exitcode=1"

exec cargo test --tests --features test-api "$@"
