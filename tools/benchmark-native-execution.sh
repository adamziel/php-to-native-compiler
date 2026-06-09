#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
usage:
  tools/benchmark-native-execution.sh [options]

Builds the PTN compiler, compiles representative PHP snippets to native
binaries, times standalone C compilation from retained generated C, and times
native execution separately.

Options:
  --runs <n>        Native execution runs per benchmark (default: 5).
  --iterations <n> Loop iterations embedded in each PHP benchmark (default: 20000).
  --output <path>  Also write the Markdown report to this path.
  --keep-temp      Keep generated PHP, C, and native binaries.
  -h, --help       Show this help.
USAGE
}

root="$(git rev-parse --show-toplevel)"
cd "$root"

runs=5
iterations=20000
output=""
keep_temp=0

require_positive_int() {
    local label="$1"
    local value="$2"
    case "$value" in
        '' | *[!0-9]*)
            echo "$label must be a positive integer" >&2
            exit 2
            ;;
    esac
    if [ "$value" -le 0 ]; then
        echo "$label must be a positive integer" >&2
        exit 2
    fi
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --runs)
            [ "$#" -ge 2 ] || {
                echo "missing value for --runs" >&2
                exit 2
            }
            require_positive_int "--runs" "$2"
            runs="$2"
            shift 2
            ;;
        --iterations)
            [ "$#" -ge 2 ] || {
                echo "missing value for --iterations" >&2
                exit 2
            }
            require_positive_int "--iterations" "$2"
            iterations="$2"
            shift 2
            ;;
        --output)
            [ "$#" -ge 2 ] || {
                echo "missing value for --output" >&2
                exit 2
            }
            output="$2"
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
        -*)
            echo "unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
        *)
            echo "unexpected argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

for tool in cargo cc date awk; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for native execution benchmarks" >&2
        exit 2
    fi
done

tmp="$(mktemp -d "${TMPDIR:-/tmp}/ptn-native-bench.XXXXXX")"
cleanup() {
    if [ "$keep_temp" -eq 1 ]; then
        echo "kept benchmark temp dir: $tmp" >&2
    else
        rm -rf "$tmp"
    fi
}
trap cleanup EXIT

now_ns() {
    local value
    value="$(date +%s%N)"
    case "$value" in
        *[!0-9]*)
            echo "date +%s%N did not produce nanoseconds on this platform" >&2
            exit 2
            ;;
    esac
    printf '%s\n' "$value"
}

elapsed_ns=0
measure_command() {
    local start
    local end
    local status
    start="$(now_ns)"
    "$@"
    status=$?
    end="$(now_ns)"
    elapsed_ns=$((end - start))
    return "$status"
}

format_ms() {
    awk -v ns="$1" 'BEGIN { printf "%.3f", ns / 1000000 }'
}

write_benchmarks() {
    local dir="$1"

    cat >"$dir/scalar_control.php" <<PHP
<?php
\$iterations = $iterations;
\$total = 0;
for (\$i = 0; \$i < \$iterations; \$i++) {
    \$term = \$i * 3 - (\$i % 7);
    if ((\$i % 5) == 0) {
        \$total += \$term;
    } elseif ((\$i % 5) == 1) {
        \$total -= \$term % 11;
    } else {
        \$total += \$term & 255;
    }
}
echo \$total, "\n";
PHP

    cat >"$dir/string_internal.php" <<PHP
<?php
\$iterations = $iterations;
\$value = "ptn";
\$total = 0;
for (\$i = 0; \$i < \$iterations; \$i++) {
    \$value = str_rot13(\$value . ":" . \$i);
    \$total += strlen(\$value);
    if (strlen(\$value) > 256) {
        \$value = substr(\$value, 0, 64);
    }
}
echo strlen(\$value), ":", \$total, ":", substr(\$value, 0, 8), "\n";
PHP

    cat >"$dir/array_foreach.php" <<PHP
<?php
\$iterations = $iterations;
\$source = ["first" => 3, 2 => 5, "02" => 7, "last" => 11];
\$total = 0;
for (\$i = 0; \$i < \$iterations; \$i++) {
    foreach (\$source as \$key => \$value) {
        \$total += \$value;
        if (\$key === "first") {
            \$total += 1;
        } elseif (\$key == 2) {
            \$total += 2;
        } else {
            \$total += strlen((string) \$key);
        }
    }
}
echo \$total, "\n";
PHP
}

bench_dir="$tmp/benchmarks"
mkdir -p "$bench_dir"
write_benchmarks "$bench_dir"

results="$tmp/results.tsv"
printf 'benchmark\tptn_compile_ms\tc_compile_ms\truntime_runs\truntime_min_ms\truntime_avg_ms\truntime_max_ms\tstdout_sample\n' >"$results"

rust_stdout="$tmp/cargo-build.stdout"
rust_stderr="$tmp/cargo-build.stderr"
if ! measure_command cargo build --quiet --bin ptn >"$rust_stdout" 2>"$rust_stderr"; then
    cat "$rust_stdout" >&2
    cat "$rust_stderr" >&2
    exit 1
fi
rust_build_ns="$elapsed_ns"
ptn_bin="$root/target/debug/ptn"

benchmarks=(scalar_control string_internal array_foreach)

for benchmark in "${benchmarks[@]}"; do
    php_file="$bench_dir/$benchmark.php"
    native_bin="$bench_dir/$benchmark.bin"
    c_file="$bench_dir/$benchmark.c"
    cc_bin="$bench_dir/$benchmark.cc.bin"

    compile_log="$bench_dir/$benchmark.ptn-compile.log"
    if ! measure_command "$ptn_bin" compile "$php_file" -o "$native_bin" --emit-c >"$compile_log" 2>&1; then
        cat "$compile_log" >&2
        exit 1
    fi
    ptn_compile_ms="$(format_ms "$elapsed_ns")"

    cc_log="$bench_dir/$benchmark.cc.log"
    if ! measure_command cc -std=c11 -Wall -Wextra -O2 "$c_file" -o "$cc_bin" -lm >"$cc_log" 2>&1; then
        cat "$cc_log" >&2
        exit 1
    fi
    c_compile_ms="$(format_ms "$elapsed_ns")"

    total_ns=0
    min_ns=0
    max_ns=0
    reference_stdout="$bench_dir/$benchmark.stdout.reference"
    for ((run = 1; run <= runs; run++)); do
        run_stdout="$bench_dir/$benchmark.stdout.$run"
        run_stderr="$bench_dir/$benchmark.stderr.$run"
        if ! measure_command "$cc_bin" >"$run_stdout" 2>"$run_stderr"; then
            cat "$run_stderr" >&2
            exit 1
        fi
        if [ -s "$run_stderr" ]; then
            echo "$benchmark emitted stderr during native run $run" >&2
            cat "$run_stderr" >&2
            exit 1
        fi
        if [ "$run" -eq 1 ]; then
            cp "$run_stdout" "$reference_stdout"
            min_ns="$elapsed_ns"
            max_ns="$elapsed_ns"
        elif ! cmp -s "$reference_stdout" "$run_stdout"; then
            echo "$benchmark stdout changed between runs" >&2
            diff -u "$reference_stdout" "$run_stdout" >&2 || true
            exit 1
        fi
        if [ "$elapsed_ns" -lt "$min_ns" ]; then
            min_ns="$elapsed_ns"
        fi
        if [ "$elapsed_ns" -gt "$max_ns" ]; then
            max_ns="$elapsed_ns"
        fi
        total_ns=$((total_ns + elapsed_ns))
    done

    avg_ns=$((total_ns / runs))
    stdout_sample="$(tr '\n' ' ' <"$reference_stdout" | awk '{ print substr($0, 1, 80) }')"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$benchmark" \
        "$ptn_compile_ms" \
        "$c_compile_ms" \
        "$runs" \
        "$(format_ms "$min_ns")" \
        "$(format_ms "$avg_ns")" \
        "$(format_ms "$max_ns")" \
        "$stdout_sample" >>"$results"
done

commit="$(git rev-parse HEAD)"
branch="$(git branch --show-current)"
origin_master="$(git rev-parse --verify origin/master 2>/dev/null || true)"
started_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
rustc_version="$(rustc --version 2>/dev/null || true)"
cargo_version="$(cargo --version 2>/dev/null || true)"
cc_version="$(cc --version 2>/dev/null | head -n 1 || true)"
kernel="$(uname -a)"
cpu_count="$(getconf _NPROCESSORS_ONLN 2>/dev/null || true)"
cpu_model=""
if [ -r /proc/cpuinfo ]; then
    cpu_model="$(awk -F: '/model name/ { sub(/^[ \t]+/, "", $2); print $2; exit }' /proc/cpuinfo)"
fi
mem_total=""
if [ -r /proc/meminfo ]; then
    mem_total="$(awk '/MemTotal/ { print $2 " " $3; exit }' /proc/meminfo)"
fi

report="$tmp/report.md"
{
    echo "# PTN Native Execution Benchmark"
    echo
    echo "- Timestamp UTC: \`$started_at\`"
    echo "- Branch: \`$branch\`"
    echo "- Commit: \`$commit\`"
    if [ -n "$origin_master" ]; then
        echo "- origin/master: \`$origin_master\`"
    fi
    echo "- Runs per benchmark: \`$runs\`"
    echo "- PHP loop iterations per benchmark: \`$iterations\`"
    echo "- Rust build elapsed: \`$(format_ms "$rust_build_ns") ms\`"
    echo "- Temp dir: \`$tmp\`$([ "$keep_temp" -eq 1 ] || printf ' (removed after run)')"
    echo
    echo "## Resource Notes"
    echo
    echo "- Kernel: \`$kernel\`"
    if [ -n "$cpu_model" ]; then
        echo "- CPU: \`$cpu_model\`"
    fi
    if [ -n "$cpu_count" ]; then
        echo "- Logical CPUs: \`$cpu_count\`"
    fi
    if [ -n "$mem_total" ]; then
        echo "- MemTotal: \`$mem_total\`"
    fi
    if [ -n "$rustc_version" ]; then
        echo "- rustc: \`$rustc_version\`"
    fi
    if [ -n "$cargo_version" ]; then
        echo "- cargo: \`$cargo_version\`"
    fi
    if [ -n "$cc_version" ]; then
        echo "- cc: \`$cc_version\`"
    fi
    echo
    echo "## Commands"
    echo
    echo "- Rust build: \`cargo build --quiet --bin ptn\`"
    echo "- PTN compile: \`target/debug/ptn compile <benchmark>.php -o <benchmark>.bin --emit-c\`"
    echo "- Standalone C compile: \`cc -std=c11 -Wall -Wextra -O2 <benchmark>.c -o <benchmark>.cc.bin -lm\`"
    echo "- Native runtime: \`<benchmark>.cc.bin\` repeated \`$runs\` times"
    echo
    echo "## Results"
    echo
    echo "| Benchmark | PTN compile incl. C (ms) | Standalone C compile (ms) | Native runtime runs | Native runtime min (ms) | Native runtime avg (ms) | Native runtime max (ms) | Stdout sample |"
    echo "| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    tail -n +2 "$results" | while IFS=$'\t' read -r benchmark ptn_compile c_compile runtime_runs runtime_min runtime_avg runtime_max stdout_sample; do
        printf '| `%s` | %s | %s | %s | %s | %s | %s | `%s` |\n' \
            "$benchmark" \
            "$ptn_compile" \
            "$c_compile" \
            "$runtime_runs" \
            "$runtime_min" \
            "$runtime_avg" \
            "$runtime_max" \
            "$stdout_sample"
    done
    echo
    echo "## Benchmark Coverage"
    echo
    echo "- \`scalar_control\`: scalar arithmetic, branches, bitwise work, and counted loops."
    echo "- \`string_internal\`: string concatenation plus internal calls \`strlen()\`, \`str_rot13()\`, and \`substr()\`."
    echo "- \`array_foreach\`: ordered array literals and key/value \`foreach\` iteration."
} >"$report"

if [ -n "$output" ]; then
    mkdir -p "$(dirname "$output")"
    cp "$report" "$output"
fi

cat "$report"
