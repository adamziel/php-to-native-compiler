# Native Runtime Memory Ownership Audit

Audit target: generated C runtime emitted from `src/backend/runtime.rs`, with
codegen ownership sites in `src/backend.rs`.

## Current Ownership Map

- `PtnValue` is a tagged union over scalar payloads plus `const char *string`
  and `PtnArray *array` (`src/backend/runtime.rs:49`). `ptn_string()` and
  `ptn_owned_string()` both produce the same `PTN_STRING` shape
  (`src/backend/runtime.rs:183`, `src/backend/runtime.rs:190`), so ownership is
  not represented in the value itself.
- `ptn_duplicate_string()` allocates C-string copies
  (`src/backend/runtime.rs:223`). `ptn_value_to_string()` always returns an
  owned duplicate for scalar string conversion (`src/backend/runtime.rs:2124`),
  while `PtnStringOperand` can borrow direct strings/static conversions or own
  formatted integer/float conversions (`src/backend/runtime.rs:108`,
  `src/backend/runtime.rs:2151`, `src/backend/runtime.rs:2171`).
- `ptn_echo()` writes scalar values directly and does not allocate for current
  scalar output (`src/backend/runtime.rs:2523`). Echo is not the major
  allocation hotspot after the direct scalar path.
- `ptn_concat()` borrows or owns converted operands, but always allocates a new
  joined string result (`src/backend/runtime.rs:2198`). Generated chained concat
  therefore allocates one owned string per binary concat node.
- Array string keys are duplicated and can be freed
  (`src/backend/runtime.rs:240`, `src/backend/runtime.rs:309`). Literal arrays
  allocate a `PtnArray`, entry vector, and optional index slots
  (`src/backend/runtime.rs:455`), but there is no `PtnArray` or `PtnValue`
  destructor.
- `ptn_array_set_entry()` replaces an existing entry value without freeing the
  overwritten payload (`src/backend/runtime.rs:437`). Array reads and foreach
  return shallow `PtnValue` aliases (`src/backend/runtime.rs:1168`,
  `src/backend/runtime.rs:1435`).
- Symbol tables duplicate names and free names/index storage on teardown
  (`src/backend/runtime.rs:491`), but they do not free stored values.
  `ptn_symbols_set()` overwrites values in place without releasing the old
  payload (`src/backend/runtime.rs:595`). `ptn_runtime_free()` only frees symbol
  tables (`src/backend/runtime.rs:768`).
- User-function frames initialize a fresh runtime symbol table and store
  by-value parameter `PtnValue`s from caller arguments (`src/backend.rs:96`,
  `src/backend.rs:113`). Frame teardown has the same symbol-table-only cleanup
  boundary (`src/backend.rs:145`).
- Generated expression statements and internal-call statements materialize a
  `PtnValue`, cast it to void, and do not emit a value cleanup
  (`src/backend.rs:231`, `src/backend.rs:271`). Runtime binary expressions and
  calls similarly create temporaries without ownership cleanup
  (`src/backend.rs:1159`, `src/backend.rs:1535`).

## Hot Spots

1. Missing value destruction is the dominant ownership bottleneck. Owned strings
   from concat, casts, string offsets, string-producing internals, and arrays
   survive until process exit or are leaked after overwrite/discard because the
   runtime cannot distinguish borrowed from owned payloads.
2. Chained concat multiplies allocations. A PHP expression such as
   `$seed . ":" . $i . ":" . $total` lowers to several `ptn_concat()` calls,
   each allocating an intermediate result before the final variable write.
3. Runtime variables and constants hold shallow values. This is fast for scalar
   aliases today, but it blocks safe freeing, array mutation, references, and
   copy-on-write because lifetime is not modeled.
4. Arrays allocate keys, entries, and indexes but have no recursive destruction
   boundary. Literal arrays and string offset reads can allocate in loops even
   when their result is discarded.
5. The recent string operand fast path avoids many input conversion duplicates,
   but helper routines still often rescan known-length operands with `strlen()`
   and allocate one fresh result per call.

## Measured Scenario

Native compiler command:

```sh
PTN_CC_OPT_LEVEL=2 target/debug/ptn compile alloc-heavy.php -o alloc-heavy --emit-c
```

Allocation-heavy PHP shape:

```php
<?php
$seed = "PtnMemoryAudit";
$total = 0;
$i = 0;
while ($i < 200000) {
    $value = $seed . ":" . $i . ":" . $total;
    $total += strlen($value);
    if (str_contains($value, "Audit")) {
        $total += strlen(str_rot13($value));
    }
    $i++;
}
echo $total, "\n";
```

Generated native output stayed `11381170`. GNU `time` samples:

| case | elapsed_s | maxrss_kb |
| --- | ---: | ---: |
| alloc-heavy run 1 | 0.28 | 39368 |
| alloc-heavy run 2 | 0.31 | 39368 |
| alloc-heavy run 3 | 0.26 | 39148 |

Same iteration count with a borrowed-string control loop using
`strlen($seed)` and `str_contains($seed, "Audit")` produced `3000000` with:

| case | elapsed_s | maxrss_kb |
| --- | ---: | ---: |
| control run 1 | 0.07 | 1768 |
| control run 2 | 0.10 | 1768 |
| control run 3 | 0.07 | 1548 |

The roughly 22x RSS difference is consistent with generated concat and
string-return temporaries being allocated repeatedly and not destroyed during
the loop.

## Recommended Slices

- `ptn-cqu.35`: Add `PtnValue` destruction for generated temporaries and
  runtime slots. This is the prerequisite for safe ownership cleanup and should
  include value/array destructors, variable overwrite cleanup, runtime teardown,
  and cleanup for discarded expression/internal-call temporaries.
- `ptn-cqu.36`: Optimize concat chains with a single-allocation string builder.
  Keep left-to-right materialization and diagnostics, but avoid one owned
  string per binary concat node.
- `ptn-cqu.37`: Thread length-aware string operands through runtime helpers.
  This is lower risk and can remove repeated `strlen()` scans in helpers whose
  callers already have `PtnStringOperand.len`.

Keep copy-on-write, references, binary-safe string storage, and broad array
mutation out of these slices unless the owning bead explicitly covers them.
