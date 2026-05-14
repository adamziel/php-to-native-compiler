# Native Runtime ABI

This document records the first runtime ABI surface for future generated native
code. It is a prerequisite for linked native execution, not a claim that native
executables are supported today.

## Current ABI Slice

`php_runtime` exposes a C-compatible scalar handoff type:

```rust
#[repr(u8)]
pub enum NativeScalarTag {
    Null = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
}

#[repr(C)]
pub struct NativeScalarValue {
    tag: NativeScalarTag,
    bool_value: u8,
    int_value: i64,
    float_value: f64,
}
```

The exported constructor symbols are:

- `phpc_native_null() -> NativeScalarValue`
- `phpc_native_bool(bool) -> NativeScalarValue`
- `phpc_native_int(i64) -> NativeScalarValue`
- `phpc_native_float(f64) -> NativeScalarValue`

The first scalar output helper symbols are:

- `phpc_native_scalar_echo_len(NativeScalarValue) -> usize`
- `phpc_native_scalar_echo_write(NativeScalarValue, *mut u8, usize) -> usize`

`phpc_native_scalar_echo_write` returns the total byte length required even when
the provided buffer is null or smaller than the output. When a non-null buffer
and nonzero capacity are supplied, it writes as many bytes as fit. This gives
future generated code a two-pass sizing/writing path without defining heap
ownership yet.

The Rust runtime can convert this ABI value back into the current interpreter
`Value` model with `NativeScalarValue::to_value()`.

## Why This Exists

Current `phpc compile --emit-ir` and `--emit-asm` mostly format scalar output
directly in generated code. That cannot scale to full PHP or WordPress
compatibility. Generated code must eventually hand PHP-shaped values to runtime
helpers for echo conversion, arithmetic/coercion, arrays, objects, references,
copy-on-write, calls, diagnostics, and host services.

This scalar ABI slice gives native lowering a small stable target before
introducing heap-owned strings, arrays, objects, error handles, symbol tables,
or linked executable commands. The scalar echo helper is the first runtime
conversion helper; generated LLVM does not call it yet.

## LLVM Helper Probe

Milestone 636 adds a deterministic IR snapshot at
`tests/fixtures/milestone636/native_runtime_scalar_echo_probe.ir`. The snapshot
declares the exported scalar echo helper symbols and includes one probe call to
`phpc_native_scalar_echo_len` so the compiler crate has a stable native-runtime
dependency artifact.

This is not production lowering. Normal `phpc compile --emit-ir` output remains
unchanged and still does not link or execute runtime helper calls. The snapshot
uses the current 64-bit test target's `usize`/pointer-width shape; a real linked
native backend still needs explicit target data layout handling before helper
calls can be emitted truthfully for all supported targets.

## Verification

The ABI slice is covered by runtime unit tests:

```sh
cargo test -p php_runtime native_scalar_abi -- --test-threads=1
cargo test -p php_runtime native_scalar_echo_helper -- --test-threads=1
cargo test -p phpc --test native_runtime_abi -- --test-threads=1
```

The tests pin:

- numeric tag discriminants for null, bool, int, and float;
- conversion from exported constructor return values into runtime `Value`;
- bool payload normalization for nonzero C-side payloads.
- scalar echo byte sizing, partial buffer writes, and null-buffer sizing.
- the deterministic compiler-side IR probe that names the exported scalar echo
  helper declarations without claiming linked native execution.

## Explicit Non-Support

This ABI does not yet provide:

- string ownership or string interning;
- arrays, objects, resources, references, or copy-on-write containers;
- runtime helper calls from normal generated LLVM IR;
- symbol tables, stack frames, call lookup, or diagnostics;
- a link command or native executable `phpc` mode;
- PHP request state, WordPress host state, or extension integration.

Those are follow-up ABI and compiler-output milestones.
