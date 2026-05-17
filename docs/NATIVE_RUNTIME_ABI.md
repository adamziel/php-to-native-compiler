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

#[repr(C)]
pub struct NativeByteBuffer {
    pub ptr: *mut u8,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
pub struct NativeStringHandle {
    ptr: *mut NativeString,
}

#[repr(C)]
pub struct NativeValueHandle {
    ptr: *mut Value,
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
- `phpc_native_scalar_echo_bytes(NativeScalarValue) -> NativeByteBuffer`
- `phpc_native_byte_buffer_from_bytes(*const u8, usize) -> NativeByteBuffer`
- `phpc_native_byte_buffer_free(NativeByteBuffer)`
- `phpc_native_string_from_bytes(*const u8, usize) -> NativeStringHandle`
- `phpc_native_string_len(NativeStringHandle) -> usize`
- `phpc_native_string_bytes(NativeStringHandle) -> *const u8`
- `phpc_native_string_clone_bytes(NativeStringHandle) -> NativeByteBuffer`
- `phpc_native_string_free(NativeStringHandle)`
- `phpc_native_value_from_string(NativeStringHandle) -> NativeValueHandle`
- `phpc_native_value_echo_bytes(NativeValueHandle) -> NativeByteBuffer`
- `phpc_native_value_echo_stdout(NativeValueHandle) -> usize`
- `phpc_native_value_free(NativeValueHandle)`

`phpc_native_scalar_echo_write` returns the total byte length required even when
the provided buffer is null or smaller than the output. When a non-null buffer
and nonzero capacity are supplied, it writes as many bytes as fit. This gives
future generated code a two-pass sizing/writing path without defining heap
ownership yet.

`phpc_native_scalar_echo_bytes` returns runtime-owned bytes for the supported
scalar echo conversion. Non-empty buffers carry the allocation pointer, length,
and capacity that must later be returned to `phpc_native_byte_buffer_free`.
Empty echo output is represented as a null pointer with zero length and zero
capacity. This owned-buffer path is ABI/probe groundwork only; normal generated
echo lowering does not call it yet.

`phpc_native_byte_buffer_from_bytes` copies a caller-provided byte slice into a
runtime-owned `NativeByteBuffer` with the same free contract. Null or
zero-length inputs return the canonical empty buffer. This is heap ownership
groundwork for future PHP string handoff only; it is not a PHP string value
handle, does not intern strings, and does not change normal generated string or
echo lowering.

`phpc_native_string_from_bytes` copies caller-provided bytes into an opaque
runtime-owned PHP string handle. Empty strings are valid handles that must be
freed; a null pointer with a nonzero length returns a null handle. The length
helper returns the byte length, the bytes helper returns a borrowed pointer that
is valid only until the handle is freed, and the clone helper returns an owned
`NativeByteBuffer` copy for generated-code handoff or probes. This is a string
handle ABI seed only: it does not intern strings, expose mutable string storage,
or change normal generated string/echo lowering.

`phpc_native_value_from_string` clones a valid UTF-8 native string handle into
an opaque runtime-owned `Value::String` handle. The source string handle remains
owned by the caller and must still be freed separately. Null string handles and
non-UTF-8 byte payloads return a null value handle until diagnostics handles and
binary PHP string values have native ABI coverage. `phpc_native_value_echo_bytes`
returns runtime-owned echo bytes for the current value handle, and
`phpc_native_value_echo_stdout` writes the current value handle's PHP echo bytes
to stdout and returns the number of bytes written. Null handles and host write
failures return zero until diagnostics handles exist. `phpc_native_value_free`
releases the value handle.

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

Milestone 1044 makes that helper signature renderer target-pointer-width aware.
The default probe still renders the current host-width snapshot, while
`native_runtime_scalar_echo_probe_ir_for_target(...)` can render explicit
32-bit or 64-bit `usize` helper signatures. The committed 32-bit snapshot lives
at `tests/fixtures/milestone637/native_runtime_scalar_echo_probe_i32.ir` and
pins `phpc_native_scalar_echo_len(...) -> i32` plus the `i32` capacity argument
for `phpc_native_scalar_echo_write(...)`. This is a prerequisite for truthful
helper-call lowering across targets; it is not a linker, runtime call emission,
or native execution path.

Milestone 1562 extends the same deterministic probe with
`%phpc.NativeByteBuffer = type { ptr, usize, usize }`, declarations for
`phpc_native_scalar_echo_bytes` and `phpc_native_byte_buffer_free`, and a small
probe function that extracts the owned buffer length before freeing it. The
probe still does not alter production `phpc compile` echo lowering or imply
linked native execution support.

Milestone 1567 adds the copied-byte ownership helper declaration
`phpc_native_byte_buffer_from_bytes(ptr, usize) -> NativeByteBuffer` plus a
target-width-aware probe function that clones a static byte payload, extracts
the owned length, and frees the buffer. Normal `phpc compile` output remains on
the existing direct `printf` scalar lowering path.

Milestone 1573 adds the opaque string handle declaration
`%phpc.NativeStringHandle = type { ptr }`, the string copy/length/borrowed-byte
pointer/clone/free helper declarations, and a target-width-aware probe function
that round-trips a static byte payload through a string handle before freeing
both the cloned buffer and the handle. The milestone also pins a CLI
`--emit-ir` fixture proving normal generated string output still uses the
existing direct `printf` path and does not call the new string helpers.

Milestone 1579 adds the opaque runtime value handle declaration
`%phpc.NativeValueHandle = type { ptr }`, a bounded
`phpc_native_value_from_string` helper that clones valid UTF-8 string handles
into runtime `Value::String` handles, value echo-byte cloning, and value-handle
freeing. The deterministic probe includes a string-handle-to-value echo path,
while normal generated string output still used the existing direct `printf`
lowering path at that milestone.

Milestone 1585 adds the first normal generated LLVM helper-call lowering path:
statement-form `print` of a direct compile-time string value is copied into a
native string handle, cloned into a runtime value handle, written through
`phpc_native_value_echo_stdout`, and then freed. The deterministic probe now
declares and calls the stdout helper for host-width and explicit 32-bit
snapshots.

Milestone 1591 extends that normal generated LLVM helper-call lowering path to
statement-form `echo` of direct compile-time string values. Each such value is
copied into a native string handle, cloned into a runtime value handle, written
through `phpc_native_value_echo_stdout`, and then freed with the same helper
sequence used by the `print` slice.

Milestone 1597 extends normal generated LLVM helper-call lowering to one
bounded dynamic string-pointer expression shape: when a selected string pointer
has a statically known set of possible PHP string values and every possible
value has the same byte length, statement-form `echo` and `print` copy the
selected pointer plus that known byte length through
`phpc_native_string_from_bytes`, clone it into a runtime value handle, write it
through `phpc_native_value_echo_stdout`, and free both handles. Mixed-length
selected string pointers still use the older direct null-terminated `printf`
fallback until native lowering can carry a selected byte length alongside the
selected pointer.

This is still not linked native execution. Dynamic string-pointer expression
output beyond the known same-byte-length selected string-pointer slice, binary
PHP string value handles beyond valid UTF-8 byte payloads, diagnostics handles,
request state, arrays, objects, resources, references, WordPress host state,
and C fallback assembly helper calls remain outside this ABI slice. The
snapshot uses the selected target's `usize`/pointer-width shape; a real linked
native backend still needs full target data layout, calling-convention
validation, runtime linking, and runtime helper emission before broader helper
calls can execute truthfully for all supported targets.

## Verification

The ABI slice is covered by runtime unit tests:

```sh
cargo test -p php_runtime native_scalar_abi -- --test-threads=1
cargo test -p php_runtime native_scalar_echo_helper -- --test-threads=1
cargo test -p php_runtime native_string_handle -- --test-threads=1
cargo test -p php_runtime native_string -- --test-threads=1
cargo test -p phpc --test native_runtime_abi -- --test-threads=1
```

The tests pin:

- numeric tag discriminants for null, bool, int, and float;
- conversion from exported constructor return values into runtime `Value`;
- bool payload normalization for nonzero C-side payloads.
- scalar echo byte sizing, partial buffer writes, and null-buffer sizing.
- owned scalar echo byte buffers, including empty null-buffer results and the
  exported free helper.
- copied runtime-owned byte buffers from caller-provided byte slices, including
  embedded NUL bytes, zero-length inputs, and null inputs.
- opaque runtime-owned string handles copied from caller-provided bytes,
  including embedded NUL bytes, empty-string handles, borrowed byte pointers,
  cloned owned-byte buffers, null non-empty inputs, and handle release.
- opaque runtime-owned value handles converted from valid UTF-8 string handles,
  including embedded NUL bytes, independent source-handle lifetime, value
  echo-byte cloning, null string handles, non-UTF-8 rejection, null-handle
  stdout echo behavior, and value-handle release.
- the deterministic compiler-side IR probe that names the exported scalar echo
  helper declarations without claiming linked native execution.
- explicit 32-bit and 64-bit `usize` IR rendering for scalar echo, owned
  byte-buffer, string-handle, value-handle, and value stdout helper
  declarations plus the probe calls.
- the normal generated `--emit-ir` CLI path for statement-form `print` and
  `echo` of direct compile-time string values through the runtime string/value
  stdout helpers.
- the normal generated `--emit-ir` CLI path for same-byte-length selected
  string-pointer `echo` and `print` expressions through the runtime
  string/value stdout helpers.

## Explicit Non-Support

This ABI does not yet provide:

- string interning, mutable string storage, binary PHP string value conversion,
  diagnostics for failed handle conversion or stdout writes, or production
  lowering of PHP strings through runtime helpers beyond the scalar echo
  owned-byte helper, copied raw-byte buffer helper, opaque copied string-handle
  helper, valid UTF-8 string-handle-to-value helper, the narrow statement-form
  direct string `echo`/`print` stdout helper path, and the narrow
  same-byte-length selected string-pointer `echo`/`print` stdout helper path;
- arrays, objects, resources, references, or copy-on-write containers;
- runtime helper calls from normal generated LLVM IR beyond direct
  compile-time string `echo`/`print` statements and same-byte-length selected
  string-pointer `echo`/`print` expressions;
- symbol tables, stack frames, call lookup, or diagnostics;
- a link command or native executable `phpc` mode;
- PHP request state, WordPress host state, or extension integration.

Those are follow-up ABI and compiler-output milestones.
