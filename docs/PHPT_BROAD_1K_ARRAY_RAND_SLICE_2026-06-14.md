# PHPT Broad 1k Array Rand Slice: 2026-06-14

Issue: `ptn-y75s`

This slice started from the current broad 1k classifier and looked for a
credible high-yield standard-array implementation. The largest single runnable
family, `array_chunk()`, is already green, so this branch implements the smaller
but contained `array_rand()` helper and records the remaining blocker.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-y75s-baseline
```

Generated manifest:

```text
.runtime/ptn-y75s-baseline/20260614T055320Z/phpt-baseline-1000.txt
```

Corpus revision:

```text
8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Top exclusions:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T055320Z.tsv
.runtime/phpt-progress/runnable-20260614T055320Z.txt
.runtime/phpt-progress/excluded-20260614T055320Z.tsv
```

## Array Chunk Check

The broad runnable set contains 32 `array_chunk()` rows. A focused run confirms
that this apparent high-yield family is already passing and is not the active
implementation blocker:

```sh
awk -F'\t' '$2=="runnable" && $1 ~ /^ext\/standard\/tests\/array\/array_chunk/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T055320Z.tsv | tools/run-phpt-manifest.sh -
```

Result:

| Selected | Runnable | Passed | Failed |
| ---: | ---: | ---: | ---: |
| 32 | 32 | 32 | 0 |

Run artifact:

```text
.runtime/phpt-progress/run-20260614T055900Z.log
```

## Implemented Helper

`array_rand()` is now registered as a generic internal helper. The runtime:

- validates that the input array is not empty;
- validates that `$num` is between 1 and the number of array elements;
- returns one selected integer/string key for `$num === 1`;
- returns a reindexed ordered array of unique selected keys for `$num > 1`;
- uses the existing runtime random index helper and array key/value conversion
  path rather than PHPT-row-specific output.

Focused broad rows:

```text
ext/standard/tests/array/array_rand.phpt
ext/standard/tests/array/array_rand_basic1.phpt
ext/standard/tests/array/array_rand_basic2.phpt
ext/standard/tests/array/array_rand_variation3.phpt
ext/standard/tests/array/array_rand_variation4.phpt
ext/standard/tests/array/array_rand_variation5.phpt
ext/standard/tests/array/array_rand_variation6.phpt
```

Command:

```sh
awk -F'\t' '$2=="runnable" && $1 ~ /^ext\/standard\/tests\/array\/array_rand/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T055320Z.tsv | tools/run-phpt-manifest.sh -
```

Result after the implementation:

| Selected | Runnable | Passed | Failed |
| ---: | ---: | ---: | ---: |
| 7 | 7 | 6 | 1 |

Run artifact:

```text
.runtime/phpt-progress/run-20260614T062947Z.log
```

## Remaining Blocker

The remaining failure is:

```text
ext/standard/tests/array/array_rand_variation6.phpt
```

The generated output has the expected `array_rand()` shape: a single key for
default/`1`, and reindexed arrays of selected keys for `3` and `6`. The mismatch
is in the heredoc-string-key expectation path, where the PHPT regex and PTN's
current heredoc escape bytes still disagree for embedded `\n`, `\t`, and `\0`
text. That blocker belongs with the existing heredoc/string-byte frontier, not
inside `array_rand()` selection semantics.

## Verification

```sh
cargo test --test compile_native compile_array_rand_to_native_binary
awk -F'\t' '$2=="runnable" && $1 ~ /^ext\/standard\/tests\/array\/array_chunk/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T055320Z.tsv | tools/run-phpt-manifest.sh -
awk -F'\t' '$2=="runnable" && $1 ~ /^ext\/standard\/tests\/array\/array_rand/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T055320Z.tsv | tools/run-phpt-manifest.sh -
```
