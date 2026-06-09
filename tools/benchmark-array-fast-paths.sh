#!/usr/bin/env bash
set -euo pipefail

iterations="${1:-250000}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

case "$iterations" in
    '' | *[!0-9]*)
        echo "iterations must be a positive integer" >&2
        exit 2
        ;;
esac

if [[ "$iterations" -lt 1 ]]; then
    echo "iterations must be a positive integer" >&2
    exit 2
fi

source_file="$tmp_dir/array-fast-paths-bench.php"
binary_file="$tmp_dir/array-fast-paths-bench-bin"

cat >"$source_file" <<PHP
<?php
\$items = [
    "k0" => 0,
    "k1" => 1,
    "k2" => 2,
    "k3" => 3,
    "k4" => 4,
    "k5" => 5,
    "k6" => 6,
    "k7" => 7,
    "k8" => 8,
    "k9" => 9,
    "k10" => 10,
    "k11" => 11,
    "k12" => 12,
    "k13" => 13,
    "k14" => 14,
    "k15" => 15,
    "nullish" => null,
    "zero" => "0",
];
\$total = 0;
for (\$i = 0; \$i < ${iterations}; \$i++) {
    \$total += count(\$items);
    if (array_key_exists("k15", \$items)) {
        \$total += \$items["k15"];
    }
    if (array_key_exists("missing", \$items)) {
        \$total += 1000000;
    }
    if (isset(\$items["k7"])) {
        \$total += \$items["k7"];
    }
    if (isset(\$items["nullish"])) {
        \$total += 1000000;
    }
    if (empty(\$items["missing"])) {
        \$total += 3;
    }
    if (empty(\$items["zero"])) {
        \$total += 5;
    }
}
echo \$total, "\n";
PHP

cargo build --quiet --manifest-path "$repo_root/Cargo.toml" --bin ptn
"$repo_root/target/debug/ptn" compile "$source_file" -o "$binary_file"

start_ns="$(date +%s%N)"
output="$("$binary_file")"
end_ns="$(date +%s%N)"
elapsed_ns=$((end_ns - start_ns))
elapsed_ms=$((elapsed_ns / 1000000))
expected=$((iterations * 48))

if [[ "$output" != "$expected" ]]; then
    printf 'unexpected output: got %s, expected %s\n' "$output" "$expected" >&2
    exit 1
fi

printf 'array fast paths native benchmark: iterations=%s predicate_calls=%s elapsed_ms=%s output=%s\n' \
    "$iterations" "$((iterations * 6))" "$elapsed_ms" "$output"
