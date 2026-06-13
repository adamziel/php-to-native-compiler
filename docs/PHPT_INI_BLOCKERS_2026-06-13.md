# PHPT INI Blockers: 2026-06-13

Issue: `ptn-j2ar` (merged through repaired bookkeeping source `ptn-x8p9`)

Broad baseline source:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

The generated 1k broad manifests used php-src revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

## Before

Baseline `.runtime/phpt-baseline/20260613T174528Z/phpt-baseline-1000.txt`
selected 1,000 broad rows, kept 430 runnable, and excluded 570. The classifier
reported one coarse INI bucket:

```text
classification.unsupported-language: rows=404
classification.unsupported-ini: rows=73
classification.unsupported-extension: rows=20
classification.unsupported-class-metadata: rows=51
classification.harness-cleanup: rows=4
classification.sapi-behavior: rows=13
classification.process-boundary: rows=3
classification.external-service: rows=1
classification.environment-assumption: rows=1
```

## Assert Probe

The biggest apparently narrow INI key was `assert.exception` with 17 broad rows.
Forced execution with classification disabled showed that simply accepting the
INI key is not a credible 25-row implementation slice:

```sh
PTN_PHPT_CLASSIFY=0 tools/run-phpt-manifest.sh /tmp/ptn-assert-exception-rows.txt
```

Result: 17 selected, 17 runnable, 5 passed, 12 failed. The failures span fatal
assertion output/stack traces, constructor dispatch, custom assertion exception
objects, `ini_set()` assertion toggling, arrow functions, anonymous classes, and
asymmetric visibility. Those are broader diagnostics, object metadata, parser,
and runtime-configuration gaps.

## After

Final rebased baseline
`.runtime/phpt-baseline/20260613T192554Z/phpt-baseline-1000.txt` selected the
same 1,000 broad rows, kept 443 runnable, and excluded 557 with opt-in SKIPIF
harness and array-internal COW classification enabled by the current base. The
former 73-row `unsupported-ini` bucket is now split by generic runtime surface:

| Category | Rows | Runtime surface |
| --- | ---: | --- |
| `unsupported-request-input-ini` | 28 | Request/input/upload SAPI state and superglobal population. |
| `unsupported-assertion-ini` | 17 | Assertion mode switching and `assert.exception`/assertion runtime behavior. |
| `unsupported-resource-limit-ini` | 15 | Zend memory-limit parsing/enforcement and memory manager boundaries. |
| `unsupported-diagnostics-ini` | 5 | Fatal backtrace, error log, and memory-leak diagnostic channels. |
| `unsupported-function-disable-ini` | 2 | Runtime function-table mutation via `disable_functions`. |
| `unsupported-opcache-ini` | 2 | Zend OPcache configuration outside PTN's native runtime. |
| `unsupported-scalar-format-ini` | 2 | Scalar/string formatting defaults beyond bounded `precision`. |
| `unsupported-host-path-ini` | 2 | Process-global host paths such as mail and temp directory settings. |

Representative row manifests were written under
`.runtime/phpt-progress/excluded-20260613T192554Z/`.

## Architecture Follow-Up

1. Add runtime configuration storage for mutable INI values with `ini_get()`,
   `ini_set()`, `ini_restore()`, startup `-d`, and reflection through one
   shared configuration table.
2. Model assertion runtime modes generically: `assert.exception`, legacy
   `assert_options()`, disabled assertions, callback/bail behavior, and fatal
   diagnostic output.
3. Introduce request/SAPI state for CLI argv, request input, uploads, and
   superglobal population before reopening request-input INI rows.
4. Add resource-limit and diagnostic channels only after PTN has a runtime
   memory/accounting boundary and richer fatal stack trace formatting.
5. Remove each classifier branch as the corresponding runtime surface becomes
   executable and measured.
