use std::fs;
use std::path::PathBuf;

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_STREAM_RESOURCE_REJECTION: &str = "LLVM stream-resource lowering rejects fopen(), fwrite(), fread(), rewind(), stream_get_contents(), feof(), ftell(), fseek(), fstat(), stream_get_meta_data(), and fclose() until native PHP resource handles, stream wrapper state, binary string byte fidelity, warning plus false recovery, references/copy-on-write, and exact native stream diagnostics exist; phpc run handles current bounded php://memory, php://temp, and local file stream resources";

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
fn stream_status_and_seek_builtins_track_memory_and_file_positions() {
    let path = temp_stream_path("phpc-stream-resource-status.txt");
    let source = format!(
        r#"<?php
$memory = fopen("php://memory", "w+");
fwrite($memory, "abcdef");
echo ftell($memory);
echo "|";
echo feof($memory) ? "eof" : "more";
echo "|";
echo fseek($memory, -3, SEEK_CUR);
echo ":";
echo ftell($memory);
echo ":";
echo fread($memory, 2);
echo ":";
echo ftell($memory);
echo ":";
echo feof($memory) ? "eof" : "more";
echo ":";
echo fseek($memory, 0, SEEK_END);
echo ":";
echo feof($memory) ? "eof" : "more";
fclose($memory);
$path = "{}";
$file = fopen($path, "w+");
fwrite($file, "cache-data");
echo "|";
echo ftell($file);
echo ":";
echo fseek($file, -4, SEEK_END);
echo ":";
echo ftell($file);
echo ":";
echo fread($file, 4);
echo ":";
echo feof($file) ? "eof" : "more";
fclose($file);
"#,
        path.display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "6|more|0:3:de:5:more:0:more|10:0:6:data:more"
    );
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_file(path);
}

#[test]
fn stream_metadata_builtins_report_bounded_memory_temp_and_file_fields() {
    let path = temp_stream_path("phpc-stream-resource-metadata.txt");
    let source = format!(
        r#"<?php
$memory = fopen("php://memory", "w+");
fwrite($memory, "abc");
$memory_meta = stream_get_meta_data($memory);
$memory_stat = fstat($memory);
echo $memory_meta["wrapper_type"];
echo ":";
echo $memory_meta["stream_type"];
echo ":";
echo $memory_meta["mode"];
echo ":";
echo $memory_meta["uri"];
echo ":";
echo $memory_stat["size"];
echo ":";
echo $memory_stat[7];
fread($memory, 10);
$eof_meta = stream_get_meta_data($memory);
echo ":";
echo $eof_meta["eof"] ? "eof" : "more";
fclose($memory);
$temp = fopen("php://temp", "w+b");
fwrite($temp, "temp-cache");
$temp_meta = stream_get_meta_data($temp);
$temp_stat = fstat($temp);
echo "|";
echo $temp_meta["stream_type"];
echo ":";
echo $temp_meta["mode"];
echo ":";
echo $temp_stat["size"];
fclose($temp);
$path = "{}";
$file = fopen($path, "w+");
fwrite($file, "plugin-cache");
$file_meta = stream_get_meta_data($file);
$file_stat = fstat($file);
echo "|";
echo $file_meta["wrapper_type"];
echo ":";
echo $file_meta["stream_type"];
echo ":";
echo $file_meta["mode"];
echo ":";
echo $file_meta["seekable"] ? "seekable" : "fixed";
echo ":";
echo $file_meta["uri"] === $path ? "same-uri" : "other-uri";
echo ":";
echo $file_stat["size"];
echo ":";
echo $file_stat[7];
echo ":";
echo $file_stat["mode"] > 0 ? "mode" : "no-mode";
fclose($file);
"#,
        path.display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "PHP:MEMORY:w+b:php://memory:3:3:eof|TEMP:w+b:10|plainfile:STDIO:w+:seekable:same-uri:12:12:mode"
    );
    assert_eq!(execution.exit_code, 0);
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

    let bad_whence =
        run_source("<?php\n$s = fopen('php://memory', 'w+'); fseek($s, 0, 9);\n").unwrap_err();
    assert_eq!(bad_whence.phase, Phase::Runtime);
    assert_eq!(bad_whence.line, 2);
    assert_eq!(bad_whence.column, 35);
    assert_eq!(
        bad_whence.message,
        "unsupported call fseek(): whence argument must be SEEK_SET, SEEK_CUR, or SEEK_END in the current subset"
    );

    let bad_stat = run_source("<?php\nfstat('not-resource');\n").unwrap_err();
    assert_eq!(bad_stat.phase, Phase::Runtime);
    assert_eq!(bad_stat.line, 2);
    assert_eq!(bad_stat.column, 1);
    assert_eq!(
        bad_stat.message,
        "unsupported call fstat(): stream argument must be resource in the current subset, got string"
    );
}

#[test]
fn emit_ir_folds_stream_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("fopen") ? "1" : "0";
echo is_callable("stream_get_contents") ? "1" : "0";
echo defined("SEEK_END") ? "1" : "0";
echo function_exists("fstat") ? "1" : "0";
echo is_callable("stream_get_meta_data") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 5, "{ir}");

    let error = emit_ir_source("<?php\nstream_get_meta_data($stream);\n").unwrap_err();
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
