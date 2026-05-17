use std::fs;
use std::path::PathBuf;

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_STREAM_RESOURCE_REJECTION: &str = "LLVM stream-resource lowering rejects fopen(), fwrite(), fread(), rewind(), stream_get_contents(), and fclose() until native PHP resource handles, stream wrapper state, binary string byte fidelity, warning plus false recovery, references/copy-on-write, and exact native stream diagnostics exist; phpc run handles current bounded php://memory, php://temp, and local file stream resources";

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
fn local_file_stream_resources_round_trip_utf8_contents() {
    let path = temp_stream_path("phpc-stream-resource-round-trip.txt");
    let source = format!(
        r#"<?php
$path = "{}";
$stream = fopen($path, "w+");
echo gettype($stream);
echo "|";
echo fwrite($stream, "core-cache");
rewind($stream);
echo "|";
echo fread($stream, 4);
echo "|";
echo stream_get_contents($stream);
echo "|";
echo fclose($stream) ? "closed" : "open";
$append = fopen($path, "a+");
fwrite($append, "-tail");
rewind($append);
echo "|";
echo stream_get_contents($append);
fclose($append);
"#,
        path.display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "resource|10|core|-cache|closed|core-cache-tail"
    );
    assert_eq!(execution.exit_code, 0);
    let contents = fs::read_to_string(&path).expect("temporary stream file is readable");
    assert_eq!(contents, "core-cache-tail");
    let _ = fs::remove_file(path);
}

#[test]
fn stream_resource_builtins_reject_forms_outside_current_subset() {
    let wrapper = run_source("<?php\nfopen('http://example.test', 'r');\n").unwrap_err();
    assert_eq!(wrapper.phase, Phase::Runtime);
    assert_eq!(wrapper.line, 2);
    assert_eq!(wrapper.column, 1);
    assert_eq!(
        wrapper.message,
        "unsupported call fopen(): only php://memory, php://temp, and local file paths are supported in the current stream subset"
    );

    let bad_mode = run_source("<?php\nfopen('php://memory', 'x');\n").unwrap_err();
    assert_eq!(bad_mode.phase, Phase::Runtime);
    assert_eq!(bad_mode.line, 2);
    assert_eq!(bad_mode.column, 1);
    assert_eq!(
        bad_mode.message,
        "unsupported call fopen(): mode \"x\" is not supported in the current stream subset"
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

fn temp_stream_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("{}-{}-{name}", std::process::id(), line!()));
    let _ = fs::remove_file(&path);
    path
}

#[test]
fn emit_asm_rejects_stream_resources_before_backend_execution() {
    let error = emit_asm_source("<?php\nstream_get_contents($stream);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STREAM_RESOURCE_REJECTION);
}
