#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
usage:
  tools/run-native-smoke-matrix.sh [--case NAME] [--emit-c] [--keep-temp]
  tools/run-native-smoke-matrix.sh --list

Builds the PTN compiler, compiles a compact matrix of PHP hot-path snippets to
native binaries, runs them, and verifies deterministic stdout with empty stderr.

Cases cover scalar loops, concat, string internals, arrays/foreach, user
functions, and comparisons.

Options:
  --case NAME    Run one matrix case.
  --emit-c       Keep generated C beside each native binary while temp files are
                 retained.
  --keep-temp    Keep generated PHP, C, stderr files, and native binaries.
  --list         Print case names and coverage.
  -h, --help     Show this help.

Set PTN_BIN=/path/to/ptn to use an already-built compiler binary.
USAGE
}

cases=(
    scalar_loop
    concat_loop
    string_internals
    array_foreach
    function_calls
    comparisons
)

coverage() {
    case "$1" in
        scalar_loop) printf 'scalar arithmetic and while/if loop control\n' ;;
        concat_loop) printf 'string concat chains and compound concat loops\n' ;;
        string_internals) printf 'strlen, substr, str_rot13, str_contains, md5\n' ;;
        array_foreach) printf 'ordered arrays, key normalization, count, foreach\n' ;;
        function_calls) printf 'user functions, call frames, func_num_args, returns\n' ;;
        comparisons) printf 'loose, strict, ordered numeric and string comparisons\n' ;;
        *) return 1 ;;
    esac
}

expected_stdout() {
    case "$1" in
        scalar_loop) printf '3885\n' ;;
        concat_loop) printf '29 ptn:0:1 0:11\n' ;;
        string_internals) printf '11 Angvir 1 22954e\n' ;;
        array_foreach) printf '16 alpha:2:4:beta:4\n' ;;
        function_calls) printf '370\n' ;;
        comparisons) printf '1255\n' ;;
        *) return 1 ;;
    esac
}

write_case() {
    local name="$1"
    local path="$2"

    case "$name" in
        scalar_loop)
            cat >"$path" <<'PHP'
<?php
$sum = 0;
$i = 0;
while ($i < 64) {
    if (($i % 3) == 0) {
        $sum += $i * 4;
    } else {
        $sum += $i - 5;
    }
    $i++;
}
echo $sum, "\n";
PHP
            ;;
        concat_loop)
            cat >"$path" <<'PHP'
<?php
$out = "ptn";
$i = 0;
while ($i < 12) {
    $out .= ":" . $i;
    $i++;
}
echo strlen($out), " ", substr($out, 0, 7), " ", substr($out, -4), "\n";
PHP
            ;;
        string_internals)
            cat >"$path" <<'PHP'
<?php
$input = "NativeSmoke";
$rot = str_rot13($input);
echo strlen($input), " ", substr($rot, 0, 6), " ", str_contains($rot, "Fz"), " ", substr(md5($input), 0, 6), "\n";
PHP
            ;;
        array_foreach)
            cat >"$path" <<'PHP'
<?php
$rows = ["alpha" => 1, "2" => 3, 4 => 5, "beta" => 7];
$total = 0;
$keys = "";
foreach ($rows as $key => $value) {
    $total += $value;
    $keys .= $key . ":";
}
echo $total, " ", $keys, count($rows), "\n";
PHP
            ;;
        function_calls)
            cat >"$path" <<'PHP'
<?php
function add3($a, $b, $c) {
    return $a + $b * $c + func_num_args();
}

function fold_limit($limit) {
    $sum = 0;
    $i = 0;
    while ($i < $limit) {
        $sum += add3($i, 2, 3);
        $i++;
    }
    return $sum;
}

echo fold_limit(20), "\n";
PHP
            ;;
        comparisons)
            cat >"$path" <<'PHP'
<?php
$hits = 0;
$i = 0;
$word = "alpha";
$other = "beta";
$numeric = "42";
while ($i < 100) {
    if ($i < 50) {
        $hits += 1;
    }
    if ($word < $other) {
        $hits += 2;
    }
    if ($numeric == 42) {
        $hits += 3;
    }
    if ($i === 42) {
        $hits += 5;
    }
    if ($word !== $other) {
        $hits += 7;
    }
    $i++;
}
echo $hits, "\n";
PHP
            ;;
        *)
            echo "unknown case: $name" >&2
            return 1
            ;;
    esac
}

case_filter=""
emit_c=0
keep_temp=0
list_only=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        --case)
            [ "$#" -ge 2 ] || {
                echo "missing value for --case" >&2
                exit 2
            }
            case_filter="$2"
            shift 2
            ;;
        --emit-c)
            emit_c=1
            shift
            ;;
        --keep-temp)
            keep_temp=1
            shift
            ;;
        --list)
            list_only=1
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

if [ "$list_only" -eq 1 ]; then
    for name in "${cases[@]}"; do
        printf '%-18s %s\n' "$name" "$(coverage "$name")"
    done
    exit 0
fi

selected=()
for name in "${cases[@]}"; do
    if [ -z "$case_filter" ] || [ "$case_filter" = "$name" ]; then
        selected+=("$name")
    fi
done

if [ "${#selected[@]}" -eq 0 ]; then
    echo "unknown case: $case_filter" >&2
    echo "available cases:" >&2
    for name in "${cases[@]}"; do
        echo "  $name" >&2
    done
    exit 2
fi

root="$(git rev-parse --show-toplevel)"
cd "$root"

tmp="$(mktemp -d "${TMPDIR:-/tmp}/ptn-native-smoke.XXXXXX")"
cleanup() {
    if [ "$keep_temp" -eq 1 ]; then
        echo "kept native smoke temp dir: $tmp" >&2
    else
        rm -rf "$tmp"
    fi
}
trap cleanup EXIT

ptn_bin="${PTN_BIN:-}"
if [ -z "$ptn_bin" ]; then
    cargo build --quiet --bin ptn
    ptn_bin="$root/target/debug/ptn"
elif [ ! -x "$ptn_bin" ]; then
    echo "PTN_BIN is not executable: $ptn_bin" >&2
    exit 2
fi

printf 'PTN native smoke matrix: cases=%s compiler=%s\n' "${#selected[@]}" "$ptn_bin"

pass_count=0
for name in "${selected[@]}"; do
    source_file="$tmp/$name.php"
    native_file="$tmp/$name-bin"
    stdout_file="$tmp/$name.stdout"
    stderr_file="$tmp/$name.stderr"
    expected_file="$tmp/$name.expected"
    write_case "$name" "$source_file"

    compile_args=(compile "$source_file" -o "$native_file")
    if [ "$emit_c" -eq 1 ]; then
        compile_args+=(--emit-c)
    fi
    "$ptn_bin" "${compile_args[@]}" >/dev/null

    set +e
    "$native_file" >"$stdout_file" 2>"$stderr_file"
    status=$?
    set -e

    expected_stdout "$name" >"$expected_file"
    if [ "$status" -ne 0 ]; then
        printf 'FAIL %-18s native exit status %s\n' "$name" "$status" >&2
        sed -n '1,40p' "$stderr_file" >&2
        exit 1
    fi
    if [ -s "$stderr_file" ]; then
        printf 'FAIL %-18s native stderr was not empty\n' "$name" >&2
        sed -n '1,40p' "$stderr_file" >&2
        exit 1
    fi
    if ! cmp -s "$expected_file" "$stdout_file"; then
        printf 'FAIL %-18s stdout mismatch\n' "$name" >&2
        diff -u --label expected --label actual "$expected_file" "$stdout_file" >&2 || true
        exit 1
    fi

    pass_count=$((pass_count + 1))
    printf 'PASS %-18s %s\n' "$name" "$(coverage "$name")"
done

printf 'native smoke matrix complete: pass=%s fail=0\n' "$pass_count"
