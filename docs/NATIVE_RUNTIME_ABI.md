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

#[repr(C)]
pub struct NativeDiagnosticHandle {
    ptr: *mut NativeDiagnostic,
}

#[repr(C)]
pub struct NativeArrayHandle {
    ptr: *mut NativeArray,
}

#[repr(C)]
pub struct NativeObjectHandle {
    ptr: *mut NativeObject,
}

#[repr(C)]
pub struct NativeResourceHandle {
    ptr: *mut NativeResource,
}

#[repr(C)]
pub struct NativeReferenceHandle {
    ptr: *mut NativeReference,
}

#[repr(C)]
pub struct NativeSymbolTableHandle {
    ptr: *mut NativeSymbolTable,
}

#[repr(C)]
pub struct NativeRequestStateHandle {
    ptr: *mut NativeRequestState,
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
- `phpc_native_value_from_scalar(NativeScalarValue) -> NativeValueHandle`
- `phpc_native_value_from_string(NativeStringHandle) -> NativeValueHandle`
- `phpc_native_value_from_string_with_diagnostic(NativeStringHandle, *mut NativeDiagnosticHandle) -> NativeValueHandle`
- `phpc_native_value_echo_bytes(NativeValueHandle) -> NativeByteBuffer`
- `phpc_native_value_echo_stdout(NativeValueHandle) -> usize`
- `phpc_native_value_free(NativeValueHandle)`
- `phpc_native_diagnostic_message_len(NativeDiagnosticHandle) -> usize`
- `phpc_native_diagnostic_message_clone_bytes(NativeDiagnosticHandle) -> NativeByteBuffer`
- `phpc_native_diagnostic_message_stderr(NativeDiagnosticHandle) -> usize`
- `phpc_native_diagnostic_free(NativeDiagnosticHandle)`
- `phpc_native_array_null() -> NativeArrayHandle`
- `phpc_native_array_is_null(NativeArrayHandle) -> bool`
- `phpc_native_array_empty() -> NativeArrayHandle`
- `phpc_native_array_len(NativeArrayHandle) -> usize`
- `phpc_native_array_append_scalar(NativeArrayHandle, NativeScalarValue) -> bool`
- `phpc_native_array_append_value(NativeArrayHandle, NativeValueHandle) -> bool`
- `phpc_native_array_read_int(NativeArrayHandle, i64) -> NativeValueHandle`
- `phpc_native_array_free(NativeArrayHandle)`
- `phpc_native_object_null() -> NativeObjectHandle`
- `phpc_native_object_is_null(NativeObjectHandle) -> bool`
- `phpc_native_resource_null() -> NativeResourceHandle`
- `phpc_native_resource_is_null(NativeResourceHandle) -> bool`
- `phpc_native_reference_null() -> NativeReferenceHandle`
- `phpc_native_reference_is_null(NativeReferenceHandle) -> bool`
- `phpc_native_symbol_table_null() -> NativeSymbolTableHandle`
- `phpc_native_symbol_table_new() -> NativeSymbolTableHandle`
- `phpc_native_symbol_table_is_null(NativeSymbolTableHandle) -> bool`
- `phpc_native_symbol_table_read_with_diagnostic(NativeSymbolTableHandle, *const u8, usize, *mut NativeDiagnosticHandle) -> NativeValueHandle`
- `phpc_native_symbol_table_write(NativeSymbolTableHandle, *const u8, usize, NativeValueHandle) -> bool`
- `phpc_native_symbol_table_free(NativeSymbolTableHandle)`
- `phpc_native_request_state_null() -> NativeRequestStateHandle`
- `phpc_native_request_state_is_null(NativeRequestStateHandle) -> bool`

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

`phpc_native_value_from_string` clones a native string handle into an opaque
runtime-owned PHP string value handle. Valid UTF-8 bytes become `Value::String`;
arbitrary byte payloads become byte-backed `Value::BinaryString`. The source
string handle remains owned by the caller and must still be freed separately.
Null string handles return a null value handle. The opt-in
`phpc_native_value_from_string_with_diagnostic` variant preserves that return
shape and, when the caller supplies a non-null diagnostic out pointer, stores a
runtime-owned diagnostic handle for null string handles or malformed raw byte
inputs such as a null byte pointer with a nonzero length. Diagnostic message
helpers expose the stable message byte length, a
runtime-owned message copy, and a bounded stderr reporting helper that writes
the diagnostic message bytes and returns the number of bytes written. Callers
must free diagnostic handles with `phpc_native_diagnostic_free` and copied
message buffers with `phpc_native_byte_buffer_free`. This diagnostic slice
covers only native string-to-value conversion failures; it does not provide a
general exception, warning, or errno channel. Binary PHP string values still
lack native ABI coverage. `phpc_native_value_echo_bytes`
returns runtime-owned echo bytes for the current value handle, and
`phpc_native_value_echo_stdout` writes the current value handle's PHP echo bytes
to stdout, flushes after a successful write, and returns the number of bytes
written. Null handles and host write failures return zero until stdout
diagnostics have ABI coverage.
`phpc_native_value_free` releases the value handle.

`NativeArrayHandle` now owns a bounded runtime `PhpArray`. The current array
ABI can allocate an empty array, report length, append a scalar value, append a
clone of an existing runtime value handle, read an integer-keyed slot as a new
runtime-owned value handle, and free the array. Null array handles make length
return `0`, append return `false`, and reads return null value handles. Missing
integer keys also return null value handles until native diagnostics have array
read coverage. `phpc_native_array_append_value` clones the input value, so the
caller still owns and must free the original value handle.

`NativeSymbolTableHandle` owns a runtime symbol table keyed by variable-name
bytes. The current helper surface can allocate a table, write a cloned runtime
value handle into a named slot, read a named slot as a new runtime-owned value
handle with diagnostics, test nullability, and free the table. The runtime
family also contains deeper path/reference operations used by generated-native
C consumers, but this does not make LLVM `--emit-ir` variable lowering general
and does not implement full PHP `$GLOBALS`, include-scope, or copy-on-write
parity for every backend.

`NativeObjectHandle`, `NativeResourceHandle`, and `NativeReferenceHandle` are
still null-only opaque handle shapes in this slice. Their exported null
constructors and predicates pin the pointer-sized C ABI forms for future
generated code and probes, but they do not allocate storage, expose PHP
object/resource/reference values, or change native lowering support.

`NativeRequestStateHandle` is also a null-only opaque handle shape in this
slice. Its constructor and predicate pin a pointer-sized request-state ABI form
for future generated SAPI/request code, but it does not allocate request
storage, expose superglobals, import host SAPI state, or change native lowering
support.

The Rust runtime can convert this ABI value back into the current interpreter
`Value` model with `NativeScalarValue::to_value()`.

## Why This Exists

Current `phpc compile --emit-ir` and `--emit-asm` mostly format scalar output
directly in generated code. That cannot scale to full PHP or WordPress
compatibility. Generated code must eventually hand PHP-shaped values to runtime
helpers for echo conversion, arithmetic/coercion, arrays, objects, references,
copy-on-write, calls, diagnostics, and host services.

This ABI gives native lowering a stable target before broadening PHP-shaped
values across every backend. Heap-owned strings, bounded arrays, diagnostics,
symbol tables, selected generated LLVM helper calls, and a small linked C path
now exist as separate slices; objects, resources, references/COW, stack frames,
request/SAPI population, and complete diagnostics remain follow-up work.

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

Milestone 2300 adds the first linked native executable path. The CLI accepts
`phpc compile <input.php> --emit-exe <output>`, builds `php_runtime` as a
static library, emits C for the current straight-line native subset, links it
with `cc`, and runs direct compile-time string `echo`/`print` output through
the runtime string/value stdout helpers. This is a bounded executable proof
only: dynamic string-pointer helper lowering remains LLVM-IR-only, and arrays,
objects, functions, references, request state, exceptions, includes, and broad
PHP coercions still do not have linked native semantics.

Milestone 2301 turns the native array handle from an empty/len/free seed into
the first useful PHP array ABI slice. The probe now declares
`phpc_native_array_append_scalar`, `phpc_native_array_append_value`, and
`phpc_native_array_read_int`, and includes a function that appends a scalar,
appends a cloned runtime value handle, reads the second slot, echoes it through
the existing value helper, and frees all owned handles. Normal generated PHP
array literals still reject in `phpc compile --emit-ir` and `--emit-asm`; this
is runtime ABI groundwork, not generated array lowering.

Milestone 2302 exposes the native symbol-table handle in the deterministic
LLVM helper probe. The probe declares the pointer-sized
`%phpc.NativeSymbolTableHandle`, null/new/null-test, named write, diagnostic
read, and free helpers, then exercises a write/read/free sequence through
`NativeValueHandle` ownership. This is shared compiler ABI visibility for the
runtime symbol-table family; it does not add one-shape LLVM production lowering
for variable assignment/readback.

Milestone 1579 adds the opaque runtime value handle declaration
`%phpc.NativeValueHandle = type { ptr }`, a bounded
`phpc_native_value_from_string` helper that cloned valid UTF-8 string handles
into runtime `Value::String` handles at that milestone, value echo-byte
cloning, and value-handle freeing. Current byte-backed PHP string support also
preserves arbitrary byte payloads as `Value::BinaryString`. The deterministic
probe includes a string-handle-to-value echo path, while normal generated
string output still used the existing direct `printf` lowering path at that
milestone.

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

Milestone 1603 closes that selected-length gap for the same bounded selected
string-pointer expression family. When the selected string pointer has a
statically known set of possible PHP string values with different byte lengths,
normal generated LLVM now emits a parallel `usize` `select` for the byte length
and passes the selected pointer plus selected length through
`phpc_native_string_from_bytes`, `phpc_native_value_from_string`,
`phpc_native_value_echo_stdout`, and the handle free helpers for statement-form
`echo` and `print`.

Milestone 1609 added the first bounded diagnostics handle slice for native
runtime ABI failures. `phpc_native_value_from_string_with_diagnostic` reports
null string-handle conversion failures through an opaque runtime-owned
diagnostic handle, and the deterministic probe includes a failing
string-to-value conversion path plus diagnostic message length, message clone,
and free calls. Later byte-backed PHP string support means non-UTF-8 payloads
now materialize as `Value::BinaryString` rather than helper failures. Normal
generated LLVM output still uses the existing non-diagnostic
`phpc_native_value_from_string` helper at that milestone and does not branch on
helper failures yet.

Milestone 1615 moves normal generated string-output lowering from the
non-diagnostic string-to-value helper to
`phpc_native_value_from_string_with_diagnostic` for the existing statement-form
compile-time string and selected string-pointer `echo`/`print` slices.
Generated LLVM now allocates a diagnostic-handle slot, initializes it to the
null handle, passes it to the conversion helper, and releases the loaded
diagnostic handle after stdout emission and value-handle release. The current
lowerable string payloads are still valid UTF-8, so this is a diagnostic
ownership/cleanup path rather than a native error-reporting or recovery path.

Milestone 1621 adds null-only opaque ABI handle shapes for future native arrays,
objects, resources, and references. The deterministic probe declares the four
handle types, calls their null constructors, and checks their null predicates.
This is an ABI shape seed only; it does not add native storage, ownership,
copy-on-write, object layout, resource tables, reference containers, or
production lowering for those PHP value kinds.

Milestone 1627 adds the first deterministic generated diagnostic branch probe.
The probe calls `phpc_native_value_from_string_with_diagnostic` with malformed
raw byte input, branches on the returned nullable `NativeValueHandle`, clones
and frees the diagnostic message only on the failure branch, and keeps the
existing success branch routed through `phpc_native_value_echo_stdout`. This
pins the LLVM control-flow shape needed before normal generated helper calls can
report runtime ABI failures. Production `phpc compile` string output still uses
the existing helper cleanup path and does not branch on or print diagnostics.

Milestone 1633 adds bounded production generated diagnostic branching/reporting
for the existing runtime string-output lowering slices. Statement-form
direct-string and selected string-pointer `echo`/`print` lowering now tests the
nullable `NativeValueHandle` returned by
`phpc_native_value_from_string_with_diagnostic`; the success branch calls
`phpc_native_value_echo_stdout`, while the failure branch loads the diagnostic
handle, reports its message through
`phpc_native_diagnostic_message_stderr`, frees the diagnostic, and rejoins the
shared cleanup that frees the value and string handles. The deterministic probe
also declares and calls the stderr helper. The current lowerable string payloads
remain valid UTF-8, so this pins generated failure-path control flow and
ownership rather than claiming linked native execution.

Milestone 1639 adds the null-only opaque
`%phpc.NativeRequestStateHandle = type { ptr }` ABI shape plus
`phpc_native_request_state_null` and
`phpc_native_request_state_is_null`. The deterministic native runtime ABI probe
declares those helpers and includes a request-state null-shape probe for both
host-width and explicit 32-bit snapshots. This is request-state ABI groundwork
only: generated `$_SERVER`, `$_COOKIE`, `$_GET`, `$_POST`, `$_REQUEST`,
`$_FILES`, and `$_SESSION` lowering still rejects before emitting misleading
native code.

This is still not linked native execution. Dynamic string-pointer expression
output beyond the known selected string-pointer slices, broader binary PHP
string operations beyond the current byte-backed value boundary, diagnostics
outside this string-to-value conversion helper, request-state
storage/population, arrays, objects, resources, references, WordPress host
state, and C fallback assembly helper calls remain outside this ABI slice. The
snapshot uses the selected target's
`usize`/pointer-width shape; a real linked native backend still needs full
target data layout, calling-convention validation, runtime linking, and runtime
helper emission before broader helper calls can execute truthfully for all
supported targets.

## Verification

The ABI slice is covered by runtime unit tests:

```sh
cargo test -p php_runtime native_scalar_abi -- --test-threads=1
cargo test -p php_runtime native_scalar_echo_helper -- --test-threads=1
cargo test -p php_runtime native_string_handle -- --test-threads=1
cargo test -p php_runtime native_string -- --test-threads=1
cargo test -p php_runtime native_diagnostic -- --test-threads=1
cargo test -p php_runtime native_container_handle -- --test-threads=1
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
- opaque runtime-owned value handles converted from string handles, including
  embedded NUL bytes, arbitrary byte payloads through byte-backed PHP string
  values, independent source-handle lifetime, value echo-byte cloning, null
  string handles, null-handle stdout echo behavior, and value-handle release.
- opaque runtime-owned diagnostic handles for string-to-value conversion
  failures, including stable message bytes for null string handles and
  malformed raw byte inputs, success-path diagnostic clearing, null diagnostic
  helper behavior, message-buffer cloning, stderr message reporting, and
  diagnostic release.
- null-only opaque array/object/resource/reference handle shapes, including
  pointer-sized layout, exported null constructors, exported null predicates,
  and deterministic compiler-side probe declarations/calls.
- a null-only opaque request-state handle shape, including pointer-sized
  layout, exported null constructor, exported null predicate, and deterministic
  compiler-side probe declarations/calls.
- a deterministic compiler-side diagnostic branch probe that tests a nullable
  value-handle return, clones/reports/frees the diagnostic message on the
  failure branch, and preserves the success branch shape through stdout echo.
- the deterministic compiler-side IR probe that names the exported scalar echo
  helper declarations without claiming linked native execution.
- explicit 32-bit and 64-bit `usize` IR rendering for scalar echo, owned
  byte-buffer, string-handle, value-handle, value stdout helper, and diagnostic
  helper declarations plus the probe calls.
- the normal generated `--emit-ir` CLI path for statement-form `print` and
  `echo` of direct compile-time string values through the runtime string/value
  stdout helpers.
- the normal generated `--emit-ir` CLI path for same-byte-length selected
  string-pointer `echo` and `print` expressions through the runtime
  string/value stdout helpers.
- the normal generated `--emit-ir` CLI path for mixed-byte-length selected
  string-pointer `echo` and `print` expressions through a selected `usize`
  byte length and the runtime string/value stdout helpers.
- the normal generated `--emit-ir` CLI path for the current runtime
  string/value stdout helper slices using
  `phpc_native_value_from_string_with_diagnostic`, initialized diagnostic
  slots, nullable value-handle branching, failure-path diagnostic stderr
  reporting, and shared handle cleanup.

## Explicit Non-Support

This ABI does not yet provide:

- string interning, mutable string storage, diagnostics beyond string-to-value
  conversion failures, stdout write diagnostics, or production lowering of PHP
  strings through runtime helpers beyond the scalar echo
  owned-byte helper, copied raw-byte buffer helper, opaque copied string-handle
  helper, string-handle-to-value helpers, the narrow
  statement-form direct string `echo`/`print` stdout helper path, and the
  narrow selected string-pointer `echo`/`print` stdout helper paths;
- object, resource, reference, or copy-on-write storage/semantics beyond the
  current opaque handle shapes and predicates;
- runtime helper calls from normal generated LLVM IR beyond direct
  compile-time string `echo`/`print` statements and the currently known
  selected string-pointer `echo`/`print` expressions;
- full symbol-table parity across every backend, stack frames, call lookup, or
  general diagnostics;
- broad native executable PHP parity beyond the current generated-C linked
  slices;
- complete PHP request-state population, WordPress host state, or extension
  integration.

Those are follow-up ABI and compiler-output milestones.
