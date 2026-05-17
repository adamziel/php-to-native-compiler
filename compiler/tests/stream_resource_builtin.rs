use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_STREAM_RESOURCE_REJECTION: &str = "LLVM stream-resource lowering rejects fopen(), fwrite(), fread(), rewind(), stream_get_contents(), and fclose() until native PHP resource handles, stream wrapper state, local file I/O, binary string byte fidelity, warning plus false recovery, references/copy-on-write, and exact native stream diagnostics exist; phpc run handles current bounded php://memory and php://temp stream resources";

#[test]
fn php_memory_and_temp_stream_resources_round_trip_buffer_contents() {
    let execution = run_source(
        r#"<?php
$memory = fopen("php://memory", "w+");
$temp = fopen("php://temp", "w+b");
echo gettype($memory);
echo "|";
echo fwrite($memory, "alpha");
echo ":";
echo fwrite($memory, "-omega", 6);
rewind($memory);
echo ":";
echo fread($memory, 5);
echo ":";
echo stream_get_contents($memory);
echo "|";
fwrite($temp, "payload");
rewind($temp);
echo stream_get_contents($temp);
echo "|";
echo fclose($memory) ? "closed" : "open";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "resource|5:6:alpha:-omega|payload|closed");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn stream_resource_builtins_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$open = "fopen";
$write = "fwrite";
$rewind = "rewind";
$contents = "stream_get_contents";
$close = "fclose";
echo function_exists($open) ? "yes" : "no";
echo "|";
echo is_callable($contents) ? "callable" : "missing";
$stream = $open("php://memory", "c+");
$write($stream, "dynamic");
$rewind($stream);
echo "|";
echo $contents($stream);
echo "|";
echo $close($stream) ? "closed" : "open";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|dynamic|closed");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn stream_resource_builtins_reject_forms_outside_current_subset() {
    let local = run_source("<?php\nfopen('local.txt', 'w+');\n").unwrap_err();
    assert_eq!(local.phase, Phase::Runtime);
    assert_eq!(local.line, 2);
    assert_eq!(local.column, 1);
    assert_eq!(
        local.message,
        "unsupported call fopen(): local file stream resources are not supported in the current subset"
    );

    let wrapper = run_source("<?php\nfopen('http://example.test', 'r');\n").unwrap_err();
    assert_eq!(wrapper.phase, Phase::Runtime);
    assert_eq!(wrapper.line, 2);
    assert_eq!(wrapper.column, 1);
    assert_eq!(
        wrapper.message,
        "unsupported call fopen(): only php://memory and php://temp are supported in the current stream subset"
    );

    let bad_stream = run_source("<?php\nfwrite('not-resource', 'x');\n").unwrap_err();
    assert_eq!(bad_stream.phase, Phase::Runtime);
    assert_eq!(bad_stream.line, 2);
    assert_eq!(bad_stream.column, 1);
    assert_eq!(
        bad_stream.message,
        "unsupported call fwrite(): stream argument must be resource in the current subset, got string"
    );

    let bad_length =
        run_source("<?php\n$s = fopen('php://memory', 'w+'); fread($s, -1);\n").unwrap_err();
    assert_eq!(bad_length.phase, Phase::Runtime);
    assert_eq!(bad_length.line, 2);
    assert_eq!(bad_length.column, 35);
    assert_eq!(
        bad_length.message,
        "unsupported call fread(): length argument must be non-negative in the current subset"
    );
}

#[test]
fn emit_ir_folds_stream_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("fopen") ? "1" : "0";
echo is_callable("stream_get_contents") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");

    let error = emit_ir_source("<?php\nfopen('php://memory', 'w+');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STREAM_RESOURCE_REJECTION);
}

#[test]
fn emit_asm_rejects_stream_resources_before_backend_execution() {
    let error = emit_asm_source("<?php\nstream_get_contents($stream);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STREAM_RESOURCE_REJECTION);
}
