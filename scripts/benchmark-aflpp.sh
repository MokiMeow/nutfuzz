#!/usr/bin/env bash
set -euo pipefail

if ! command -v afl-fuzz >/dev/null 2>&1 ||
   ! command -v afl-clang-fast >/dev/null 2>&1; then
    echo "SKIP: AFL++ tools are not installed"
    exit 0
fi

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
build_dir="$repo_dir/build/aflpp"
input_dir="$build_dir/input"
output_dir="$build_dir/output"
seconds=${NUTFUZZ_AFL_SECONDS:-10}

rm -rf "$build_dir"
mkdir -p "$input_dir" "$output_dir"
printf 'AAAAAAA' > "$input_dir/seed"

afl-clang-fast -O2 -Wall -Wextra -Werror \
    "$repo_dir/bench/afl_target.c" -o "$build_dir/afl-target"

AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 \
AFL_SKIP_CPUFREQ=1 \
afl-fuzz -V "$seconds" -i "$input_dir" -o "$output_dir" \
    -- "$build_dir/afl-target"

stats="$output_dir/default/fuzzer_stats"
if [[ -f "$stats" ]]; then
    grep -E '^(execs_done|execs_per_sec|saved_crashes|edges_found)' "$stats" || true
fi
echo "AFL++ owned-target output: $output_dir"
