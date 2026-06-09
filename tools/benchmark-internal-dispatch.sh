#!/usr/bin/env bash
set -euo pipefail

iterations="${1:-200000}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

source_file="$tmp_dir/internal-dispatch-bench.php"
binary_file="$tmp_dir/internal-dispatch-bench-bin"

cat >"$source_file" <<PHP
<?php
\$total = 0;
for (\$i = 0; \$i < ${iterations}; \$i++) {
    \$total += strlen("dispatch");
    \$total += abs(-42);
    \$total += intval("123");
    \$total += ord("A");
    \$total += str_contains("lookup", "up");
}
echo \$total, "\n";
PHP

cargo build --quiet --manifest-path "$repo_root/Cargo.toml"
"$repo_root/target/debug/ptn" compile "$source_file" -o "$binary_file"

start_ns="$(date +%s%N)"
output="$("$binary_file")"
end_ns="$(date +%s%N)"
elapsed_ns=$((end_ns - start_ns))
elapsed_ms=$((elapsed_ns / 1000000))
expected=$((iterations * 239))

if [[ "$output" != "$expected" ]]; then
    printf 'unexpected output: got %s, expected %s\n' "$output" "$expected" >&2
    exit 1
fi

printf 'internal dispatch native benchmark: iterations=%s calls=%s elapsed_ms=%s output=%s\n' \
    "$iterations" "$((iterations * 5))" "$elapsed_ms" "$output"
