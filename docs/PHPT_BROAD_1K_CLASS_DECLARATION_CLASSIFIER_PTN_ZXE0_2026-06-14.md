# PHPT Broad 1k Class Declaration Classifier: 2026-06-14 ptn-zxe0

Issue: `ptn-zxe0`

This records the `ptn-zxe0` classifier evidence against the class declaration
taxonomy that existed before the later `ptn-gkvr` split. The original branch
moved 78 broad 1k rows out of the older `unsupported-language` bucket into
`unsupported-class-declaration`.

That behavior is already integrated on `master` through the later broad
language split and the `ptn-bo7q` class declaration category evidence. The
superseded aggregate category was:

| Aggregate bucket | Rows |
| --- | ---: |
| `unsupported-class-declaration` | 78 |

The current `ptn-gkvr` category split is:

| Generic blocker | Rows |
| --- | ---: |
| Trait declarations | 25 |
| Interface declarations | 23 |
| Interface implementation checks | 15 |
| Anonymous class syntax (`new class`) | 15 |

The committed focused manifest is:

```text
tools/phpt-class-declaration-frontier-manifest.txt
```

Aggregate evidence is documented in
`PHPT_BROAD_1K_CLASS_DECLARATION_CATEGORY_PTN_BO7Q_2026-06-14.md`, and the
current explicit split is documented in
`PHPT_BROAD_1K_CLASS_DECLARATION_SPLIT_PTN_GKVR_2026-06-14.md`. This note exists
so the older `ptn-zxe0` merge request is integrated without reverting to the
stale broad `unsupported-language` bucket accounting.

## Validation

The classifier test suite now covers representative class declaration blockers:
anonymous classes, interface declarations, interface implementation checks, and
trait declarations.

Focused replay:

```text
PHPT_PROGRESS_DIR=.runtime/ptn-zxe0-class-declaration-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
```
