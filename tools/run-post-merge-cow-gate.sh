#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
usage:
  tools/run-post-merge-cow-gate.sh [--case NAME] [--emit-c] [--keep-temp]
  tools/run-post-merge-cow-gate.sh --list

Builds the PTN compiler, runs a compact post-merge copy-on-write gate, and
prints numeric pass/fail counts. Supported cases compile to native binaries and
must match PHP stdout/stderr exactly. Notice cases compile to native binaries
and match explicit PTN stdout/stderr expectations. Unsupported reference cases
must fail with explicit compiler diagnostics.

Options:
  --case NAME    Run one oracle or diagnostic case.
  --emit-c       Keep generated C beside native binaries while temp files are
                 retained.
  --keep-temp    Keep generated PHP, stdout/stderr files, and native binaries.
  --list         Print case names and coverage.
  -h, --help     Show this help.

Set PTN_BIN=/path/to/ptn to use an already-built compiler binary.
Set PHP_BIN=/path/to/php to choose the PHP oracle binary.
USAGE
}

oracle_cases=(
    foreach_value_snapshot_appends
    foreach_value_nested_local_mutation
    foreach_by_ref_mutates_slots
    foreach_by_ref_live_appends
    function_return_array_temp
    function_return_nested_slot
    nested_array_child_copy
    string_offset_alias_detach
    function_return_string_offset
    array_element_reference_after_copy
    typed_by_reference_return_separation
    by_reference_return_boundaries
)

notice_cases=(
    reference_assignment_from_call_result_value_fallback
)

diagnostic_cases=(
    unsupported_foreach_reference_key
    unsupported_foreach_destructuring
    unsupported_by_reference_return
    unsupported_recursive_array_append_self
    unsupported_same_array_append_element_reference
    unsupported_same_array_element_reference_assignment
    unsupported_recursive_array_literal_self
    unsupported_recursive_array_literal_keyed_self
    unsupported_recursive_array_literal_nested_self
    unsupported_recursive_array_element_literal_self
    unsupported_same_array_literal_element_reference
    unsupported_same_array_element_literal_element_reference
)

coverage() {
    case "$1" in
        foreach_value_snapshot_appends)
            printf 'by-value foreach snapshot with source appends\n'
            ;;
        foreach_value_nested_local_mutation)
            printf 'by-value foreach nested element writes detach loop values\n'
            ;;
        foreach_by_ref_mutates_slots)
            printf 'by-reference foreach mutates source slots\n'
            ;;
        foreach_by_ref_live_appends)
            printf 'by-reference foreach visits live appends\n'
            ;;
        function_return_array_temp)
            printf 'function-returned array temporaries detach on caller writes\n'
            ;;
        function_return_nested_slot)
            printf 'function-returned nested slots detach through read temporaries\n'
            ;;
        nested_array_child_copy)
            printf 'nested array child copies detach from shared outers\n'
            ;;
        string_offset_alias_detach)
            printf 'string offset writes detach shared aliases\n'
            ;;
        function_return_string_offset)
            printf 'function-returned strings detach on offset writes\n'
            ;;
        array_element_reference_after_copy)
            printf 'array element references and copied siblings preserve COW\n'
            ;;
        typed_by_reference_return_separation)
            printf 'typed by-reference returns separate reference-bound values\n'
            ;;
        by_reference_return_boundaries)
            printf 'by-reference return aliases, separation, array slots, locals, and typed coercion\n'
            ;;
        reference_assignment_from_call_result_value_fallback)
            printf 'notice: non-reference call results assigned by reference fall back to value writes\n'
            ;;
        unsupported_foreach_reference_key)
            printf 'diagnostic: foreach key binding cannot be by reference\n'
            ;;
        unsupported_foreach_destructuring)
            printf 'diagnostic: foreach destructuring remains explicit unsupported behavior\n'
            ;;
        unsupported_by_reference_return)
            printf 'diagnostic: non-lvalue by-reference returns remain explicit unsupported behavior\n'
            ;;
        unsupported_recursive_array_append_self)
            printf 'diagnostic: array append by reference to itself remains explicit unsupported behavior\n'
            ;;
        unsupported_same_array_append_element_reference)
            printf 'diagnostic: appending a same-array element reference remains explicit unsupported behavior\n'
            ;;
        unsupported_same_array_element_reference_assignment)
            printf 'diagnostic: same-array element reference assignment remains explicit unsupported behavior\n'
            ;;
        unsupported_recursive_array_literal_self)
            printf 'diagnostic: array literal reference to assigned variable remains explicit unsupported behavior\n'
            ;;
        unsupported_recursive_array_literal_keyed_self)
            printf 'diagnostic: keyed array literal reference to assigned variable remains explicit unsupported behavior\n'
            ;;
        unsupported_recursive_array_literal_nested_self)
            printf 'diagnostic: nested array literal reference to assigned variable remains explicit unsupported behavior\n'
            ;;
        unsupported_recursive_array_element_literal_self)
            printf 'diagnostic: array-element literal reference to containing array remains explicit unsupported behavior\n'
            ;;
        unsupported_same_array_literal_element_reference)
            printf 'diagnostic: array literal reference to a same-array element remains explicit unsupported behavior\n'
            ;;
        unsupported_same_array_element_literal_element_reference)
            printf 'diagnostic: array-element literal reference to a same-array element remains explicit unsupported behavior\n'
            ;;
        *)
            return 1
            ;;
    esac
}

write_oracle_case() {
    local name="$1"
    local path="$2"

    case "$name" in
        foreach_value_snapshot_appends)
            cat >"$path" <<'PHP'
<?php
$items = [1, 2];
foreach ($items as $value) {
    echo $value, "\n";
    $items[] = $value + 10;
}
echo "count=", count($items), "\n";
PHP
            ;;
        foreach_value_nested_local_mutation)
            cat >"$path" <<'PHP'
<?php
$rows = [["v" => 1], ["v" => 2]];
foreach ($rows as $row) {
    $row["v"] += 10;
    $row[] = "local";
}
echo $rows[0]["v"], ":", $rows[1]["v"], ":", count($rows[0]), "\n";
PHP
            ;;
        foreach_by_ref_mutates_slots)
            cat >"$path" <<'PHP'
<?php
$items = ["a" => 1, "b" => 2];
foreach ($items as $key => &$value) {
    echo $key, "=", $value, "\n";
    $value += 10;
}
unset($value);
echo $items["a"], ":", $items["b"], "\n";
PHP
            ;;
        foreach_by_ref_live_appends)
            cat >"$path" <<'PHP'
<?php
$items = [1, 2];
$seen = 0;
foreach ($items as &$value) {
    echo $value, "\n";
    if ($seen < 2) {
        $items[] = $value + 10;
    }
    $seen++;
}
unset($value);
echo "count=", count($items), "\n";
PHP
            ;;
        function_return_array_temp)
            cat >"$path" <<'PHP'
<?php
function identity_array($value) {
    return $value;
}
$base = ["v" => 1];
$returned = identity_array($base);
$returned["v"] = 2;
$returned[] = "tail";
echo $base["v"], ":", $returned["v"], ":", count($base), ":", count($returned), "\n";
PHP
            ;;
        function_return_nested_slot)
            cat >"$path" <<'PHP'
<?php
function make_rows() {
    return [["v" => 10], ["v" => 20]];
}
$row = make_rows()[1];
$copy = $row;
$copy["v"] = 99;
$copy[] = "tail";
echo $row["v"], ":", $copy["v"], ":", count($copy), "\n";
PHP
            ;;
        nested_array_child_copy)
            cat >"$path" <<'PHP'
<?php
$outer = [["x" => 1], ["x" => 2]];
$copy = $outer;
$child = $copy[0];
$child["x"] = 9;
$copy[1]["x"] = 8;
echo $outer[0]["x"], ":", $child["x"], ":", $outer[1]["x"], ":", $copy[1]["x"], "\n";
PHP
            ;;
        string_offset_alias_detach)
            cat >"$path" <<'PHP'
<?php
$text = "abcd";
$copy = $text;
$copy[1] = "X";
echo $text, ":", $copy, "\n";
PHP
            ;;
        function_return_string_offset)
            cat >"$path" <<'PHP'
<?php
function make_text($prefix) {
    return $prefix . "bc";
}
$text = make_text("a");
$copy = $text;
$copy[2] = "Z";
echo $text, ":", $copy, "\n";
PHP
            ;;
        array_element_reference_after_copy)
            cat >"$path" <<'PHP'
<?php
$items = [1, 2];
$copy = $items;
$ref =& $items[0];
$ref = 7;
$copy[1] = 8;
echo $items[0], ":", $copy[0], ":", $items[1], ":", $copy[1], ":", $ref, "\n";
PHP
            ;;
        typed_by_reference_return_separation)
            cat >"$path" <<'PHP'
<?php
function test_value(&$value): string {
    return $value;
}

function &test_reference(int &$value): string {
    return $value;
}

$value = 123;
echo test_value($value), ":", gettype($value), "\n";
echo test_reference($value), ":", gettype($value), ":", $value, "\n";
PHP
            ;;
        by_reference_return_boundaries)
            cat >"$path" <<'PHP'
<?php
function &id(&$value) {
    return $value;
}

function &slot(&$items) {
    return $items["k"];
}

function &local_box() {
    $local = 41;
    return $local;
}

function &as_string(&$value): string {
    return $value;
}

function wrap_copy(&$value) {
    return id($value);
}

$value = 1;
$alias =& id($value);
$alias = 2;
echo $value, "|", $alias, "\n";

$copy = id($value);
$copy = 3;
echo $value, "|", $copy, "\n";

$items = ["k" => 4];
$slot =& slot($items);
$slot = 5;
echo $items["k"], "|", $slot, "\n";

$local =& local_box();
$local = 42;
echo $local, "\n";

$typed = 123;
$typed_alias =& as_string($typed);
echo gettype($typed), ":", $typed, "|", gettype($typed_alias), ":", $typed_alias, "\n";
$typed_alias = "abc";
echo gettype($typed), ":", $typed, "\n";

$wrapped = 7;
$wrapped_copy = wrap_copy($wrapped);
$wrapped_copy = 8;
echo $wrapped, "|", $wrapped_copy, "\n";
PHP
            ;;
        *)
            echo "unknown oracle case: $name" >&2
            return 1
            ;;
    esac
}

write_diagnostic_case() {
    local name="$1"
    local path="$2"

    case "$name" in
        unsupported_foreach_reference_key)
            cat >"$path" <<'PHP'
<?php
$items = [1];
foreach ($items as &$key => $value) {
    echo $value;
}
PHP
            ;;
        unsupported_foreach_destructuring)
            cat >"$path" <<'PHP'
<?php
foreach ([[1]] as [$value]) {
    echo $value;
}
PHP
            ;;
        unsupported_by_reference_return)
            cat >"$path" <<'PHP'
<?php
function &make_ref() {
    return 1;
}
PHP
            ;;
        unsupported_recursive_array_append_self)
            cat >"$path" <<'PHP'
<?php
$array = [];
$array[] =& $array;
PHP
            ;;
        unsupported_same_array_append_element_reference)
            cat >"$path" <<'PHP'
<?php
$array = [1];
$array[] =& $array[0];
PHP
            ;;
        unsupported_same_array_element_reference_assignment)
            cat >"$path" <<'PHP'
<?php
$array = [1, 2];
$array[0] =& $array[1];
PHP
            ;;
        unsupported_recursive_array_literal_self)
            cat >"$path" <<'PHP'
<?php
$array = [&$array];
PHP
            ;;
        unsupported_recursive_array_literal_keyed_self)
            cat >"$path" <<'PHP'
<?php
$array = ["self" => &$array];
PHP
            ;;
        unsupported_recursive_array_literal_nested_self)
            cat >"$path" <<'PHP'
<?php
$array = [[&$array]];
PHP
            ;;
        unsupported_recursive_array_element_literal_self)
            cat >"$path" <<'PHP'
<?php
$array = [];
$array[] = [&$array];
PHP
            ;;
        unsupported_same_array_literal_element_reference)
            cat >"$path" <<'PHP'
<?php
$array = [&$array[0]];
PHP
            ;;
        unsupported_same_array_element_literal_element_reference)
            cat >"$path" <<'PHP'
<?php
$array = [];
$array[] = [&$array[0]];
PHP
            ;;
        *)
            echo "unknown diagnostic case: $name" >&2
            return 1
            ;;
    esac
}

write_notice_case() {
    local name="$1"
    local path="$2"

    case "$name" in
        reference_assignment_from_call_result_value_fallback)
            cat >"$path" <<'PHP'
<?php
function make_value() {
    return 1;
}
$ref =& make_value();
echo $ref, "\n";
$items = [];
$items["slot"] =& make_value();
echo $items["slot"], "\n";
PHP
            ;;
        *)
            echo "unknown notice case: $name" >&2
            return 1
            ;;
    esac
}

expected_notice_stdout() {
    case "$1" in
        reference_assignment_from_call_result_value_fallback)
            cat <<'OUT'
Notice: Only variables should be assigned by reference in ptn on line 5
1
Notice: Only variables should be assigned by reference in ptn on line 8
1
OUT
            ;;
        *)
            return 1
            ;;
    esac
}

expected_diagnostic() {
    case "$1" in
        unsupported_foreach_reference_key)
            printf 'Key element cannot be a reference\n'
            ;;
        unsupported_foreach_destructuring)
            printf 'foreach destructuring is unsupported\n'
            ;;
        unsupported_by_reference_return)
            printf 'by-reference return requires a variable or array element\n'
            ;;
        unsupported_recursive_array_append_self)
            printf 'recursive array references are unsupported\n'
            ;;
        unsupported_same_array_append_element_reference)
            printf 'same-array element references are unsupported\n'
            ;;
        unsupported_same_array_element_reference_assignment)
            printf 'same-array element references are unsupported\n'
            ;;
        unsupported_recursive_array_literal_self)
            printf 'recursive array references are unsupported\n'
            ;;
        unsupported_recursive_array_literal_keyed_self)
            printf 'recursive array references are unsupported\n'
            ;;
        unsupported_recursive_array_literal_nested_self)
            printf 'recursive array references are unsupported\n'
            ;;
        unsupported_recursive_array_element_literal_self)
            printf 'recursive array references are unsupported\n'
            ;;
        unsupported_same_array_literal_element_reference)
            printf 'same-array element references are unsupported\n'
            ;;
        unsupported_same_array_element_literal_element_reference)
            printf 'same-array element references are unsupported\n'
            ;;
        *)
            return 1
            ;;
    esac
}

contains_case() {
    local needle="$1"
    shift
    local item
    for item in "$@"; do
        [ "$item" = "$needle" ] && return 0
    done
    return 1
}

run_oracle_case() {
    local name="$1"
    local source_file="$tmp/$name.php"
    local native_file="$tmp/$name-bin"
    local php_stdout="$tmp/$name.php.stdout"
    local php_stderr="$tmp/$name.php.stderr"
    local native_stdout="$tmp/$name.native.stdout"
    local native_stderr="$tmp/$name.native.stderr"
    local compile_stdout="$tmp/$name.compile.stdout"
    local compile_stderr="$tmp/$name.compile.stderr"

    write_oracle_case "$name" "$source_file"

    set +e
    "$php_bin" "$source_file" >"$php_stdout" 2>"$php_stderr"
    local php_status=$?
    set -e
    if [ "$php_status" -ne 0 ]; then
        printf 'FAIL oracle %-42s PHP exited with %s\n' "$name" "$php_status" >&2
        sed -n '1,40p' "$php_stderr" >&2
        return 1
    fi

    local compile_args=(compile "$source_file" -o "$native_file")
    if [ "$emit_c" -eq 1 ]; then
        compile_args+=(--emit-c)
    fi

    set +e
    "$ptn_bin" "${compile_args[@]}" >"$compile_stdout" 2>"$compile_stderr"
    local compile_status=$?
    set -e
    if [ "$compile_status" -ne 0 ]; then
        printf 'FAIL oracle %-42s native compile exited with %s\n' "$name" "$compile_status" >&2
        sed -n '1,40p' "$compile_stderr" >&2
        return 1
    fi

    set +e
    "$native_file" >"$native_stdout" 2>"$native_stderr"
    local native_status=$?
    set -e
    if [ "$native_status" -ne 0 ]; then
        printf 'FAIL oracle %-42s native exited with %s\n' "$name" "$native_status" >&2
        sed -n '1,40p' "$native_stderr" >&2
        return 1
    fi
    if ! cmp -s "$php_stdout" "$native_stdout"; then
        printf 'FAIL oracle %-42s stdout diverged from PHP oracle\n' "$name" >&2
        diff -u --label php --label native "$php_stdout" "$native_stdout" >&2 || true
        return 1
    fi
    if ! cmp -s "$php_stderr" "$native_stderr"; then
        printf 'FAIL oracle %-42s stderr diverged from PHP oracle\n' "$name" >&2
        diff -u --label php --label native "$php_stderr" "$native_stderr" >&2 || true
        return 1
    fi

    printf 'PASS oracle %-42s %s\n' "$name" "$(coverage "$name")"
    return 0
}

run_notice_case() {
    local name="$1"
    local source_file="$tmp/$name.php"
    local native_file="$tmp/$name-bin"
    local compile_stdout="$tmp/$name.compile.stdout"
    local compile_stderr="$tmp/$name.compile.stderr"
    local native_stdout="$tmp/$name.native.stdout"
    local native_stderr="$tmp/$name.native.stderr"
    local expected_stdout="$tmp/$name.expected.stdout"
    local expected_stderr="$tmp/$name.expected.stderr"

    write_notice_case "$name" "$source_file"
    expected_notice_stdout "$name" >"$expected_stdout"
    : >"$expected_stderr"

    set +e
    "$ptn_bin" compile "$source_file" -o "$native_file" >"$compile_stdout" 2>"$compile_stderr"
    local compile_status=$?
    set -e
    if [ "$compile_status" -ne 0 ]; then
        printf 'FAIL notice %-42s native compile exited with %s\n' "$name" "$compile_status" >&2
        sed -n '1,40p' "$compile_stderr" >&2
        return 1
    fi

    set +e
    "$native_file" >"$native_stdout" 2>"$native_stderr"
    local native_status=$?
    set -e
    if [ "$native_status" -ne 0 ]; then
        printf 'FAIL notice %-42s native exited with %s\n' "$name" "$native_status" >&2
        sed -n '1,40p' "$native_stderr" >&2
        return 1
    fi
    if ! cmp -s "$expected_stdout" "$native_stdout"; then
        printf 'FAIL notice %-42s stdout diverged from expectation\n' "$name" >&2
        diff -u --label expected --label native "$expected_stdout" "$native_stdout" >&2 || true
        return 1
    fi
    if ! cmp -s "$expected_stderr" "$native_stderr"; then
        printf 'FAIL notice %-42s stderr diverged from expectation\n' "$name" >&2
        diff -u --label expected --label native "$expected_stderr" "$native_stderr" >&2 || true
        return 1
    fi

    printf 'PASS notice %-42s %s\n' "$name" "$(coverage "$name")"
    return 0
}

run_diagnostic_case() {
    local name="$1"
    local source_file="$tmp/$name.php"
    local native_file="$tmp/$name-bin"
    local compile_stdout="$tmp/$name.compile.stdout"
    local compile_stderr="$tmp/$name.compile.stderr"
    local expected="$tmp/$name.expected"
    local combined="$tmp/$name.compile.combined"

    write_diagnostic_case "$name" "$source_file"
    expected_diagnostic "$name" >"$expected"

    set +e
    "$ptn_bin" compile "$source_file" -o "$native_file" >"$compile_stdout" 2>"$compile_stderr"
    local compile_status=$?
    set -e
    cat "$compile_stdout" "$compile_stderr" >"$combined"
    if [ "$compile_status" -eq 0 ]; then
        printf 'FAIL diagnostic %-38s compiled successfully; expected diagnostic\n' "$name" >&2
        return 1
    fi
    if ! grep -Fq "$(cat "$expected")" "$combined"; then
        printf 'FAIL diagnostic %-38s missing expected diagnostic\n' "$name" >&2
        printf 'expected substring: %s\n' "$(cat "$expected")" >&2
        sed -n '1,40p' "$combined" >&2
        return 1
    fi

    printf 'PASS diagnostic %-38s %s\n' "$name" "$(coverage "$name")"
    return 0
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
    for name in "${oracle_cases[@]}"; do
        printf 'oracle     %-42s %s\n' "$name" "$(coverage "$name")"
    done
    for name in "${notice_cases[@]}"; do
        printf 'notice     %-42s %s\n' "$name" "$(coverage "$name")"
    done
    for name in "${diagnostic_cases[@]}"; do
        printf 'diagnostic %-42s %s\n' "$name" "$(coverage "$name")"
    done
    exit 0
fi

selected_oracle=()
selected_notice=()
selected_diagnostic=()
for name in "${oracle_cases[@]}"; do
    if [ -z "$case_filter" ] || [ "$case_filter" = "$name" ]; then
        selected_oracle+=("$name")
    fi
done
for name in "${notice_cases[@]}"; do
    if [ -z "$case_filter" ] || [ "$case_filter" = "$name" ]; then
        selected_notice+=("$name")
    fi
done
for name in "${diagnostic_cases[@]}"; do
    if [ -z "$case_filter" ] || [ "$case_filter" = "$name" ]; then
        selected_diagnostic+=("$name")
    fi
done

if [ -n "$case_filter" ] \
    && ! contains_case "$case_filter" "${oracle_cases[@]}" \
    && ! contains_case "$case_filter" "${notice_cases[@]}" \
    && ! contains_case "$case_filter" "${diagnostic_cases[@]}"; then
    echo "unknown case: $case_filter" >&2
    echo "available cases:" >&2
    for name in "${oracle_cases[@]}"; do
        echo "  $name" >&2
    done
    for name in "${notice_cases[@]}"; do
        echo "  $name" >&2
    done
    for name in "${diagnostic_cases[@]}"; do
        echo "  $name" >&2
    done
    exit 2
fi

root="$(git rev-parse --show-toplevel)"
cd "$root"

tmp="$(mktemp -d "${TMPDIR:-/tmp}/ptn-post-merge-cow.XXXXXX")"
cleanup() {
    if [ "$keep_temp" -eq 1 ]; then
        echo "kept post-merge COW gate temp dir: $tmp" >&2
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

php_bin="${PHP_BIN:-php}"
if ! command -v "$php_bin" >/dev/null 2>&1; then
    echo "PHP_BIN is not executable or not on PATH: $php_bin" >&2
    exit 2
fi

printf 'PTN post-merge COW gate: oracle_cases=%s notice_cases=%s diagnostic_cases=%s compiler=%s php=%s\n' \
    "${#selected_oracle[@]}" "${#selected_notice[@]}" "${#selected_diagnostic[@]}" "$ptn_bin" "$php_bin"

oracle_pass=0
oracle_fail=0
notice_pass=0
notice_fail=0
diagnostic_pass=0
diagnostic_fail=0

for name in "${selected_oracle[@]}"; do
    if run_oracle_case "$name"; then
        oracle_pass=$((oracle_pass + 1))
    else
        oracle_fail=$((oracle_fail + 1))
    fi
done

for name in "${selected_notice[@]}"; do
    if run_notice_case "$name"; then
        notice_pass=$((notice_pass + 1))
    else
        notice_fail=$((notice_fail + 1))
    fi
done

for name in "${selected_diagnostic[@]}"; do
    if run_diagnostic_case "$name"; then
        diagnostic_pass=$((diagnostic_pass + 1))
    else
        diagnostic_fail=$((diagnostic_fail + 1))
    fi
done

printf 'post-merge COW gate complete: oracle_pass=%s oracle_fail=%s notice_pass=%s notice_fail=%s diagnostics_pass=%s diagnostics_fail=%s\n' \
    "$oracle_pass" "$oracle_fail" "$notice_pass" "$notice_fail" "$diagnostic_pass" "$diagnostic_fail"

if [ "$oracle_fail" -ne 0 ] || [ "$notice_fail" -ne 0 ] || [ "$diagnostic_fail" -ne 0 ]; then
    exit 1
fi
