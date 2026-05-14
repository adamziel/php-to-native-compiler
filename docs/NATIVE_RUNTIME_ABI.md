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
or linked executable commands.

## Verification

The ABI slice is covered by runtime unit tests:

```sh
cargo test -p php_runtime native_scalar_abi -- --test-threads=1
```

The tests pin:

- numeric tag discriminants for null, bool, int, and float;
- conversion from exported constructor return values into runtime `Value`;
- bool payload normalization for nonzero C-side payloads.

## Explicit Non-Support

This ABI does not yet provide:

- string ownership or string interning;
- arrays, objects, resources, references, or copy-on-write containers;
- runtime helper calls from generated LLVM IR;
- symbol tables, stack frames, call lookup, or diagnostics;
- a link command or native executable `phpc` mode;
- PHP request state, WordPress host state, or extension integration.

Those are follow-up ABI and compiler-output milestones.
