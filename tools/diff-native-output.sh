#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
usage:
  tools/diff-native-output.sh <input.php>
  tools/diff-native-output.sh --snippet '<?php echo "Hello\n";'
  tools/diff-native-output.sh -

Compiles the PHP input with `ptn compile`, runs the native binary, runs the
same input with system `php`, and compares stdout, stderr, and exit status.

Options:
  --file <path>       Read PHP input from a file.
  --snippet <source>  Read PHP input from the argument.
  --emit-c            Keep generated C beside the native binary while temp
                      files are retained.
  --keep-temp         Keep temporary files and print their location.
  -h, --help          Show this help.
USAGE
}

root="$(git rev-parse --show-toplevel)"
cd "$root"

input_arg=""
snippet=""
read_stdin=0
emit_c=0
keep_temp=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        --file)
            [ "$#" -ge 2 ] || {
                echo "missing value for --file" >&2
                exit 2
            }
            input_arg="$2"
            shift 2
            ;;
        --snippet)
            [ "$#" -ge 2 ] || {
                echo "missing value for --snippet" >&2
                exit 2
            }
            snippet="$2"
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
        -)
            read_stdin=1
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
            if [ -n "$input_arg" ]; then
                echo "only one input may be provided" >&2
                exit 2
            fi
            input_arg="$1"
            shift
            ;;
    esac
done

input_modes=0
[ -n "$input_arg" ] && input_modes=$((input_modes + 1))
[ -n "$snippet" ] && input_modes=$((input_modes + 1))
[ "$read_stdin" -eq 1 ] && input_modes=$((input_modes + 1))
if [ "$input_modes" -ne 1 ]; then
    echo "provide exactly one input: a file, --snippet, or -" >&2
    usage >&2
    exit 2
fi

if ! command -v php >/dev/null 2>&1; then
    echo "system php is required for differential telemetry" >&2
    exit 2
fi

tmp="$(mktemp -d "${TMPDIR:-/tmp}/ptn-diff.XXXXXX")"
cleanup() {
    if [ "$keep_temp" -eq 1 ]; then
        echo "kept telemetry temp dir: $tmp" >&2
    else
        rm -rf "$tmp"
    fi
}
trap cleanup EXIT

input="$input_arg"
label="$input_arg"
if [ -n "$snippet" ]; then
    input="$tmp/snippet.php"
    label="snippet"
    printf '%s' "$snippet" >"$input"
elif [ "$read_stdin" -eq 1 ]; then
    input="$tmp/stdin.php"
    label="stdin"
    cat >"$input"
fi

native="$tmp/native"
compile_args=(compile "$input" -o "$native")
if [ "$emit_c" -eq 1 ]; then
    compile_args+=(--emit-c)
fi

cargo run --quiet --bin ptn -- "${compile_args[@]}"

php_stdout="$tmp/php.stdout"
php_stderr="$tmp/php.stderr"
native_stdout="$tmp/native.stdout"
native_stderr="$tmp/native.stderr"

set +e
php "$input" >"$php_stdout" 2>"$php_stderr"
php_status=$?
"$native" >"$native_stdout" 2>"$native_stderr"
native_status=$?
set -e

matched=1
if ! cmp -s "$php_stdout" "$native_stdout"; then
    matched=0
    echo "stdout mismatch for $label" >&2
    diff -u --label php.stdout --label native.stdout "$php_stdout" "$native_stdout" || true
fi

if ! cmp -s "$php_stderr" "$native_stderr"; then
    matched=0
    echo "stderr mismatch for $label" >&2
    diff -u --label php.stderr --label native.stderr "$php_stderr" "$native_stderr" || true
fi

if [ "$php_status" -ne "$native_status" ]; then
    matched=0
    echo "status mismatch for $label: php=$php_status native=$native_status" >&2
fi

if [ "$matched" -eq 1 ]; then
    echo "PASS $label"
else
    echo "FAIL $label" >&2
    exit 1
fi
