# PHPT String/Scalar Alias Slice: 2026-06-13

Scope: `ptn-550s.5` broad PHPT rows for string-offset COW, quiet string
offset diagnostics, scalar offset boundaries, concat-assignment aliasing, and
mixed scalar/array reference blockers.

## Runtime Change

- Quiet string-offset `isset()`/`empty()` probes now still emit PHP-style
  lossy float-to-int deprecations while suppressing ordinary offset warnings.
- Resource keys in quiet string-offset probes are treated as missing, matching
  PHP `isset()`/`empty()` behavior.
- Shared float-to-int precision deprecation formatting uses enough significant
  digits for constants such as `M_PI` while preserving ordinary decimal
  spellings used by existing reducers.

## Focused Counts

Manifest: `tools/phpt-string-scalar-alias-manifest.txt`

| Bucket | Rows | Pass | Classified |
| --- | ---: | ---: | ---: |
| quiet-string-offsets | 2 | 2 | 0 |
| numeric-and-scalar-offsets | 7 | 7 | 0 |
| string-offset-cow | 4 | 4 | 0 |
| concat-and-ref-aliasing | 10 | 10 | 0 |
| classified-blockers | 12 | 0 | 12 |
| **Total** | **35** | **23** | **12** |

Classified blockers: unsupported heredoc/nowdoc syntax (5), unsupported ini
requirements (4), typed property metadata (2), and unavailable `zend_test`
extension coverage (1).

## Before/After Evidence

- Before broad candidate run:
  `.runtime/phpt-progress/manifest-20260613T185953Z.txt`
  selected 44 rows, ran 36, excluded 8, passed 11, failed 25.
  `Zend/tests/empty_str_offset.phpt` and
  `Zend/tests/isset/isset_str_offset.phpt` both failed.
- After focused manifest run:
  `.runtime/phpt-progress/manifest-20260613T193922Z.txt`
  selected 35 rows, ran 23, excluded 12, passed 23, failed 0 after rebasing.
  The same quiet string-offset rows pass.

## Verification

- `cargo test --test compile_native compile_quiet_string_offset_isset_empty_conversions_to_native_binary -- --nocapture`
- `printf '%s\n' 'Zend/tests/empty_str_offset.phpt' 'Zend/tests/isset/isset_str_offset.phpt' | tools/run-phpt-manifest.sh -`
- `tools/run-phpt-manifest.sh tools/phpt-string-scalar-alias-manifest.txt`
