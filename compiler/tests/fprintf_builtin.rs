use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_STREAM_RESOURCE_REJECTION: &str = "LLVM stream-resource lowering rejects fopen(), tmpfile(), stream_context_create(), stream_context_get_options(), stream_context_get_params(), stream_context_get_default(), stream_context_set_default(), stream_context_set_option(), stream_context_set_options(), stream_context_set_params(), fwrite()/fputs(), fgetc(), fgets(), fgetcsv(), fputcsv(), fscanf(), fread(), rewind(), stream_get_contents(), fpassthru(), stream_copy_to_stream(), stream_filter_append(), stream_is_local(), stream_supports_lock(), flock(), feof(), ftell(), fseek(), fstat(), stream_get_meta_data(), stream_get_wrappers(), stream_wrapper_register(), stream_wrapper_unregister(), stream_wrapper_restore(), fclose(), dir(), opendir(), readdir(), rewinddir(), closedir(), glob(), is_uploaded_file(), and move_uploaded_file() until native PHP resource handles, stream wrapper state, stream context state, stream wrapper registry state, stream filter state, stream lock state, upload provenance state, directory/glob state, binary string byte fidelity, warning plus false recovery, references/copy-on-write, and exact native stream diagnostics exist; phpc run handles current bounded php://memory, php://temp, php://input, data://, local file stream resources, stream wrapper capability metadata, stream context resources, selected read filters, local directory handles, bounded local glob patterns, and PHPC_FILES upload provenance";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn fprintf_and_vfprintf_write_streams_and_return_byte_lengths() {
    let execution = run_source(
        r#"<?php
$stream = fopen("php://memory", "w+");
$left = fprintf($stream, "%s:%04d:%x\n", "id", 7, 255);
$right = vfprintf($stream, "%s/%s/%b", ["right", "left", 5]);
rewind($stream);
echo stream_get_contents($stream), "|", $left, "|", $right;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "id:0007:ff\nright/left/101|11|14");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn fprintf_and_vfprintf_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$stream = fopen("php://memory", "w+");
$call = "fprintf";
echo function_exists($call) ? "yes" : "no";
echo "|", is_callable("vfprintf") ? "callable" : "missing";
echo "|", $call($stream, "%+d", 7);
$call = "vfprintf";
echo "|", $call($stream, "%X", [255]);
rewind($stream);
echo "|", stream_get_contents($stream);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|2|2|+7FF");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn fprintf_and_vfprintf_reject_unsupported_operands() {
    let arity = runtime_error("<?php\nfprintf();\n");
    assert_eq!(arity.line, 2);
    assert_eq!(arity.column, 1);
    assert_eq!(
        arity.message,
        "arity mismatch for fprintf(): expected at least 2 argument(s), got 0"
    );

    let stream = runtime_error("<?php\nfprintf(\"not-stream\", \"%s\", \"x\");\n");
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 1);
    assert_eq!(
        stream.message,
        "unsupported call fprintf(): stream argument must be resource in the current subset, got string"
    );

    let scalar_format = run_source(
        r#"<?php
$stream = fopen("php://memory", "w+");
$length = fprintf($stream, 42, "x");
rewind($stream);
echo stream_get_contents($stream), "|", $length;
"#,
    )
    .unwrap();
    assert_eq!(scalar_format.stdout, "42|2");
    assert_eq!(scalar_format.exit_code, 0);

    let values = runtime_error(
        r#"<?php
$stream = fopen("php://memory", "w+");
vfprintf($stream, "%s", "not-array");
"#,
    );
    assert_eq!(values.line, 3);
    assert_eq!(values.column, 1);
    assert_eq!(
        values.message,
        "unsupported call vfprintf(): values argument must be array in the current subset, got string"
    );
}

#[test]
fn emit_ir_recognizes_fprintf_metadata_but_rejects_runtime_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("fprintf") ? "1" : "0";
echo is_callable("vfprintf") ? "1" : "0";
"#,
    )
    .unwrap();
    assert!(ir.contains("@.str"), "{ir}");

    let error = emit_ir_source(
        r#"<?php
$stream = fopen("php://memory", "w+");
fprintf($stream, "%s", "x");
"#,
    )
    .unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_STREAM_RESOURCE_REJECTION);
}
