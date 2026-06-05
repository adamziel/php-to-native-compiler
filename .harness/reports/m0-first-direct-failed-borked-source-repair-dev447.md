# M0 First Direct FAILED/BORKED Source Repair Selector

Lane: 151
Developer: developer-447
Generated: 2026-06-05T19:11:35Z
Scope: read-only selector and handoff. No compiler, runtime, harness, or docs source files were edited.

## Decision

Open the first source repair lane on internal/core readonly property write and
unset diagnostics for already-supported core objects, starting with `Directory`
and `BcMath\Number`.

This is the first direct source candidate because it is:

- a direct `FAILED` bucket from the blocked `221205Z` candidate, not an
  `ABSENT` result-normalization problem;
- product behavior, unlike the three `BORKED` SKIPIF constant rows;
- non-late-priority PHP compatibility work, with no dependency on `eval` or
  variable variables;
- narrower than the full readonly/property-hook surface and backed by existing
  object metadata, Directory, and BcMath test files in this repo.

Do not include property hooks, interface property satisfaction, SKIPIF wrapper
constants, readonly class semantics, or userland readonly declarations in this
first implementation lane.

## Exact PHPT Rows

Primary source-repair rows:

| Status | PHPT row | Reason |
| --- | --- | --- |
| `FAILED` | `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt` | Internal `Directory::$handle` write/unset must throw PHP readonly-property errors and preserve the resource. |
| `FAILED` | `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt` | Internal `Directory::$path` write/unset must throw PHP readonly-property errors and preserve the path. |
| `FAILED` | `php-src/ext/bcmath/tests/number/properties_write_error.phpt` | Internal `BcMath\Number::$value` and `::$scale` writes must throw readonly-property errors. |
| `FAILED` | `php-src/ext/bcmath/tests/number/properties_unset.phpt` | Internal `BcMath\Number::$value` and `::$scale` unsets must throw readonly-property errors. |

Wider postcheck rows only after the primary four rows are green:

- `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt`
- `php-src/ext/date/tests/DatePeriod_properties2.phpt`
- `php-src/ext/xmlreader/tests/014.phpt`
- `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt`
- `php-src/Zend/tests/attributes/deprecated/property_readonly_002.phpt`

Explicitly excluded from this lane:

- `php-src/Zend/tests/property_hooks/gh19548.phpt`
- `php-src/Zend/tests/property_hooks/gh19548_002.phpt`
- `php-src/Zend/tests/attributes/override/properties_08.phpt`
- `php-src/Zend/tests/readonly_classes/*.phpt`
- `php-src/Zend/tests/readonly_props/readonly_trait_mismatch.phpt`
- the three `BORKED` SKIPIF rows for `INTL_ICU_VERSION`,
  `ZEND_THREAD_SAFE`, and `PCRE_JIT_SUPPORT`

## Precheck Commands

Run these only after the accepted and blocked-candidate release `phpc` binaries
are restored or rebuilt. The historical paths are documented as missing in
`.harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md`, so these
commands are intentionally exact but currently gated on executable `PHPC_BIN`
paths.

```sh
REPLAY_ROOT=/tmp/phpt-focused-replay-lane151-internal-readonly
PHP_SRC=/home/claude/php-src-phpt
WRAPPER=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
ROWLIST="$REPLAY_ROOT/internal-readonly-direct.tests"

install -d -m 700 "$REPLAY_ROOT"
cat > "$ROWLIST" <<'EOF'
/home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt
/home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt
/home/claude/php-src-phpt/ext/bcmath/tests/number/properties_write_error.phpt
/home/claude/php-src-phpt/ext/bcmath/tests/number/properties_unset.phpt
EOF
```

Accepted-baseline replay:

```sh
ACCEPTED_OUT="$REPLAY_ROOT/accepted"
export PHPC_BIN=/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
export TEST_PHP_EXECUTABLE="$WRAPPER"
export TEST_PHP_ARGS=
export TMPDIR="$ACCEPTED_OUT/tmp"
export TEMP="$TMPDIR"
export TMP="$TMPDIR"
export PHPC_PHPT_TIMEOUT_SECONDS=55
export PHPC_PHPT_KILL_AFTER_SECONDS=5
export PHPT_SYSTEM_PHP=php
export TEST_PHP_SRCDIR="$PHP_SRC"
export NO_INTERACTION=1

install -d -m 700 "$ACCEPTED_OUT" "$TMPDIR" "$ACCEPTED_OUT/phpt-tmp"
cd "$PHP_SRC"
php run-tests.php -q -n \
  -p "$TEST_PHP_EXECUTABLE" \
  -r "$ROWLIST" \
  -W "$ACCEPTED_OUT/results.txt" \
  -s "$ACCEPTED_OUT/run-tests.log" \
  --no-color \
  --set-timeout 65 \
  --temp-source "$PHP_SRC" \
  --temp-target "$ACCEPTED_OUT/phpt-tmp" \
  > "$ACCEPTED_OUT/stdout.log" \
  2> "$ACCEPTED_OUT/stderr.log"
```

Blocked-candidate replay:

```sh
CANDIDATE_OUT="$REPLAY_ROOT/candidate"
export PHPC_BIN=/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
export TEST_PHP_EXECUTABLE="$WRAPPER"
export TEST_PHP_ARGS=
export TMPDIR="$CANDIDATE_OUT/tmp"
export TEMP="$TMPDIR"
export TMP="$TMPDIR"
export PHPC_PHPT_TIMEOUT_SECONDS=55
export PHPC_PHPT_KILL_AFTER_SECONDS=5
export PHPT_SYSTEM_PHP=php
export TEST_PHP_SRCDIR="$PHP_SRC"
export NO_INTERACTION=1

install -d -m 700 "$CANDIDATE_OUT" "$TMPDIR" "$CANDIDATE_OUT/phpt-tmp"
cd "$PHP_SRC"
php run-tests.php -q -n \
  -p "$TEST_PHP_EXECUTABLE" \
  -r "$ROWLIST" \
  -W "$CANDIDATE_OUT/results.txt" \
  -s "$CANDIDATE_OUT/run-tests.log" \
  --no-color \
  --set-timeout 65 \
  --temp-source "$PHP_SRC" \
  --temp-target "$CANDIDATE_OUT/phpt-tmp" \
  > "$CANDIDATE_OUT/stdout.log" \
  2> "$CANDIDATE_OUT/stderr.log"
```

Implementation should proceed only if replay confirms these rows remain direct
semantic failures. If any row becomes absent or BORKED, route that row back to
M0 result collection or wrapper/environment repair.

## Owned Files For The Repair Lane

- `runtime/src/lib.rs`
  - Add explicit readonly metadata for internal properties, preferably on
    `PhpPropertyMetadata` and materialized `ObjectProperty`.
  - Mark `BcMath\Number::$value`, `BcMath\Number::$scale`,
    `Directory::$path`, and `Directory::$handle` readonly.
  - Enforce readonly writes, unsets, and reference binding through the shared
    object-property APIs rather than ad hoc call-site checks.
- `compiler/src/interpreter.rs`
  - Preserve assignment/unset call-site behavior while surfacing the runtime's
    PHP-shaped readonly diagnostics through `RuntimeError`.
  - Keep `phpc compile --emit-ir` / `--emit-asm` rejection behavior intact for
    object/property lowering that is still unsupported.
- `compiler/tests/standard_directory_glob_builtins.rs`
  - Add a focused `Directory` run test that writes and unsets `path` and
    `handle`, checks the exact error messages, and verifies the original values
    still work.
- `compiler/tests/bcmath_builtin.rs`
  - Add a focused `BcMath\Number` run test that writes and unsets `value` and
    `scale`, checks the exact error messages, and verifies reads still report
    the original number and scale.
- runtime unit tests and `compiler/tests/syntax_boundaries.rs`
  - Add metadata coverage proving the selected core properties are readonly.
  - Keep userland readonly declarations covered as an unsupported parser
    boundary.
- `docs/PROGRESS.md` and `docs/SUPPORT.md`
  - Update only after behavior and tests pass. State the bounded support claim
    as internal/core readonly-property diagnostics for the selected properties,
    not general readonly properties.

## Focused Verification Commands

Expected Rust tests for the implementation patch:

```sh
cargo test -p phpc --test standard_directory_glob_builtins directory_core_properties_reject_write_and_unset_as_readonly -- --test-threads=1
cargo test -p phpc --test bcmath_builtin bcmath_number_properties_reject_write_and_unset_as_readonly -- --test-threads=1
cargo test -p php_runtime core_class_table_marks_selected_internal_properties_readonly -- --test-threads=1
cargo test -p phpc --test syntax_boundaries unsupported_readonly_property_declarations_have_stable_parse_errors -- --test-threads=1
```

Expected CLI exercise path:

```sh
cargo run -p phpc -- test tests/fixtures/milestone-internal-readonly-properties
cargo run -p phpc -- test --compare-php tests/fixtures/milestone-internal-readonly-properties
cargo run -p phpc -- compile tests/fixtures/milestone-internal-readonly-properties/internal_readonly_property_diagnostics.php --emit-ir
```

The compile command should continue to reject unsupported object/property
native lowering instead of emitting misleading IR or assembly.

Postcheck PHPT replay should use the same `ROWLIST` command shape, first with
the primary four rows, then with the wider postcheck rows after the primary
rows pass.

## Unsupported Edges To Name

The implementation lane must keep these edges explicit in docs and tests:

- userland readonly property/class declarations remain unsupported parser
  boundaries;
- readonly class semantics and trait readonly-property compatibility remain
  out of scope;
- property hooks and interface property satisfaction remain a separate lane;
- asymmetric `private(set)` / `protected(set)` property visibility remains out
  of scope except that readonly diagnostics must not leak those words for the
  selected rows;
- readonly references, indirect modification, append, nested array writes, and
  reflection metadata are not claimed unless separately implemented and tested;
- `DatePeriod`, `XMLReader`, and `Deprecated` readonly diagnostics are wider
  postchecks, not part of the initial support claim;
- native lowering remains a rejection boundary for unsupported object/property
  forms.

## Evidence Sources

- `.harness/reports/221205Z-direct-failed-borked-triage.md`
- `.harness/reports/focused-replay-cookbook.md`
- `.harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md`
- `runtime/src/lib.rs` core class metadata for `BcMath\Number` and `Directory`
- `compiler/src/interpreter.rs` object assignment/unset call paths
- `compiler/tests/bcmath_builtin.rs`
- `compiler/tests/standard_directory_glob_builtins.rs`
- `compiler/tests/syntax_boundaries.rs`
