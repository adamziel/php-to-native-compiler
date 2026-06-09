#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
usage:
  tools/bench-native-execution.sh [--runs N] [--keep-temp]

Builds the PTN compiler, compiles three representative PHP microbenchmarks to
native binaries, rebuilds the retained generated C, and times native execution
separately from those build steps.

Options:
  --runs N       Number of native runtime samples per benchmark. Default: 5.
  --keep-temp    Keep generated PHP, C, and native binaries and print the path.
  -h, --help     Show this help.
USAGE
}

root="$(git rev-parse --show-toplevel)"
cd "$root"

runs="${PTN_BENCH_RUNS:-5}"
keep_temp=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        --runs)
            [ "$#" -ge 2 ] || {
                echo "missing value for --runs" >&2
                exit 2
            }
            runs="$2"
            shift 2
            ;;
        --keep-temp)
            keep_temp=1
            shift
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

case "$runs" in
    '' | *[!0-9]*)
        echo "--runs must be a positive integer" >&2
        exit 2
        ;;
esac

if [ "$runs" -lt 1 ]; then
    echo "--runs must be a positive integer" >&2
    exit 2
fi

now_ns() {
    local value
    value="$(date +%s%N)"
    case "$value" in
        *N)
            echo "date +%s%N is required for sub-second benchmark timing" >&2
            exit 2
            ;;
    esac
    printf '%s\n' "$value"
}

format_ns() {
    awk -v ns="$1" 'BEGIN { printf "%.6f", ns / 1000000000 }'
}

measure_ns() {
    local start end
    start="$(now_ns)"
    "$@"
    end="$(now_ns)"
    printf '%s\n' "$((end - start))"
}

pipe_join_ns() {
    local result=""
    local value
    for value in "$@"; do
        if [ -n "$result" ]; then
            result="${result}, "
        fi
        result="${result}$(format_ns "$value")"
    done
    printf '%s\n' "$result"
}

table_escape() {
    local value
    value="$(tr '\n' ' ' | sed 's/[[:space:]]*$//')"
    value="${value//|/\\|}"
    printf '%s\n' "$value"
}

tmp="$(mktemp -d "${TMPDIR:-/tmp}/ptn-bench.XXXXXX")"
cleanup() {
    if [ "$keep_temp" -eq 1 ]; then
        echo "kept benchmark temp dir: $tmp" >&2
    else
        rm -rf "$tmp"
    fi
}
trap cleanup EXIT

cat >"$tmp/scalar_loop.php" <<'PHP'
<?php
$sum = 0;
$i = 0;
while ($i < 80000) {
    if (($i % 5) == 0) {
        $sum += $i * 3;
    } else {
        $sum += $i - 7;
    }
    $i++;
}
echo $sum, "\n";
PHP

cat >"$tmp/string_work.php" <<'PHP'
<?php
$seed = "PtnNative";
$out = "";
$i = 0;
while ($i < 1200) {
    $chunk = str_rot13($seed . $i);
    $out .= $chunk . ":" . strlen($chunk) . "|";
    $i++;
}
echo strlen($out), " ", substr(md5($out), 0, 8), "\n";
PHP

cat >"$tmp/array_foreach.php" <<'PHP'
<?php
$rows = ["alpha" => 1, "beta" => 2, "gamma" => 3, 4 => 5, "6" => 8, "omega" => 13];
$total = 0;
$passes = 0;
while ($passes < 12000) {
    foreach ($rows as $key => $value) {
        if ($key === "beta") {
            $total += $value * 3;
        } else {
            $total += $value;
        }
    }
    $passes++;
}
echo $total, "\n";
PHP

cc_bin="${CC:-cc}"
ptn_bin="$root/target/debug/ptn"
commit="$(git rev-parse HEAD)"
origin_master="$(git rev-parse origin/master 2>/dev/null || true)"
generated_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
host="$(uname -srmo 2>/dev/null || uname -a)"
cpu="$(awk -F: '/model name/ { sub(/^[ \t]+/, "", $2); print $2; exit }' /proc/cpuinfo 2>/dev/null || true)"
cores="$(getconf _NPROCESSORS_ONLN 2>/dev/null || printf 'unknown')"
memory="$(awk '/MemTotal/ { print $2 " " $3; exit }' /proc/meminfo 2>/dev/null || true)"
rustc_version="$(rustc --version 2>/dev/null || printf 'unknown')"
cargo_version="$(cargo --version 2>/dev/null || printf 'unknown')"
cc_version="$("$cc_bin" --version 2>/dev/null | head -n 1 || printf 'unknown')"

cargo_build_ns="$(measure_ns cargo build --quiet --bin ptn)"

echo "# PTN Native Execution Benchmark Baseline"
echo
echo "- Generated: $generated_at"
echo "- Commit: $commit"
if [ -n "$origin_master" ]; then
    echo "- origin/master: $origin_master"
fi
echo "- Host: $host"
if [ -n "$cpu" ]; then
    echo "- CPU: $cpu"
fi
echo "- Online cores: $cores"
if [ -n "$memory" ]; then
    echo "- Memory: $memory"
fi
echo "- rustc: $rustc_version"
echo "- cargo: $cargo_version"
echo "- C compiler: $cc_version"
echo "- Runtime samples per benchmark: $runs"
echo
echo "## Commands"
echo
echo "- Rust compiler build: \`cargo build --quiet --bin ptn\`"
echo "- PTN native build: \`target/debug/ptn compile <bench.php> -o <native> --emit-c\`"
echo "- Generated C rebuild: \`${cc_bin} -std=c11 -Wall -Wextra -O2 <bench.c> -o <native-cc> -lm\`"
echo "- Native runtime: \`<native>\` repeated $runs times; table reports best plus all samples."
echo
echo "## Compiler Build"
echo
echo "| step | seconds |"
echo "| --- | ---: |"
echo "| cargo build --bin ptn | $(format_ns "$cargo_build_ns") |"
echo
echo "## Native Benchmarks"
echo
echo "| benchmark | coverage | ptn_compile_s | cc_rebuild_s | runtime_best_s | runtime_samples_s | stdout |"
echo "| --- | --- | ---: | ---: | ---: | --- | --- |"

run_benchmark() {
    local label="$1"
    local coverage="$2"
    local input="$3"
    local native="$tmp/$label"
    local native_cc="$tmp/$label-cc"
    local c_source="$tmp/$label.c"
    local stdout_file="$tmp/$label.stdout"
    local stderr_file="$tmp/$label.stderr"
    local expected_stdout=""
    local runtime_samples_ns=()
    local best_ns=""
    local elapsed_ns
    local sample
    local stdout_text

    elapsed_ns="$(measure_ns "$ptn_bin" compile "$input" -o "$native" --emit-c)"
    if [ ! -f "$c_source" ]; then
        echo "expected generated C source $c_source" >&2
        exit 1
    fi

    local cc_elapsed_ns
    cc_elapsed_ns="$(measure_ns "$cc_bin" -std=c11 -Wall -Wextra -O2 "$c_source" -o "$native_cc" -lm)"

    for ((sample = 1; sample <= runs; sample++)); do
        local start end current_stdout
        start="$(now_ns)"
        "$native" >"$stdout_file" 2>"$stderr_file"
        end="$(now_ns)"
        if [ -s "$stderr_file" ]; then
            echo "$label emitted stderr during benchmark run:" >&2
            sed -n '1,40p' "$stderr_file" >&2
            exit 1
        fi
        current_stdout="$(cat "$stdout_file")"
        if [ "$sample" -eq 1 ]; then
            expected_stdout="$current_stdout"
        elif [ "$current_stdout" != "$expected_stdout" ]; then
            echo "$label stdout changed across benchmark runs" >&2
            exit 1
        fi
        local runtime_ns="$((end - start))"
        runtime_samples_ns+=("$runtime_ns")
        if [ -z "$best_ns" ] || [ "$runtime_ns" -lt "$best_ns" ]; then
            best_ns="$runtime_ns"
        fi
    done

    stdout_text="$(printf '%s' "$expected_stdout" | table_escape)"
    printf '| %s | %s | %s | %s | %s | %s | `%s` |\n' \
        "$label" \
        "$coverage" \
        "$(format_ns "$elapsed_ns")" \
        "$(format_ns "$cc_elapsed_ns")" \
        "$(format_ns "$best_ns")" \
        "$(pipe_join_ns "${runtime_samples_ns[@]}")" \
        "$stdout_text"
}

run_benchmark "scalar_loop" "scalar arithmetic and braced control flow" "$tmp/scalar_loop.php"
run_benchmark "string_work" "string concatenation plus strlen/str_rot13/md5/substr" "$tmp/string_work.php"
run_benchmark "array_foreach" "ordered array literal and key/value foreach" "$tmp/array_foreach.php"
