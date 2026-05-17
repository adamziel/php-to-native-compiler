use std::fs;
use std::path::PathBuf;

use php_compiler::error::Phase;
use php_compiler::interpreter::{run_program_with_options, RunOptions};
use php_compiler::{emit_asm_source, emit_ir_source, parse, run_source};

const LLVM_STREAM_RESOURCE_REJECTION: &str = "LLVM stream-resource lowering rejects fopen(), stream_context_create(), stream_context_get_options(), stream_context_get_default(), stream_context_set_default(), stream_context_set_option(), fwrite(), fread(), rewind(), stream_get_contents(), feof(), ftell(), fseek(), fstat(), stream_get_meta_data(), fclose(), opendir(), readdir(), rewinddir(), closedir(), is_uploaded_file(), and move_uploaded_file() until native PHP resource handles, stream wrapper state, stream context state, directory handle state, upload provenance state, binary string byte fidelity, warning plus false recovery, references/copy-on-write, and exact native stream diagnostics exist; phpc run handles current bounded php://memory, php://temp, php://input, local file stream resources, stream context resources, local directory handles, and PHPC_FILES upload provenance";

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
fn php_input_stream_resource_reads_seeded_request_body() {
    let program = parse(
        r#"<?php
$input = fopen("php://input", "rb");
$meta = stream_get_meta_data($input);
echo gettype($input);
echo "|";
echo $meta["wrapper_type"];
echo ":";
echo $meta["stream_type"];
echo ":";
echo $meta["mode"];
echo ":";
echo $meta["uri"];
echo "|";
echo fread($input, 7);
echo ":";
echo ftell($input);
echo ":";
echo stream_get_contents($input);
echo ":";
echo feof($input) ? "eof" : "more";
rewind($input);
echo "|";
echo fread($input, 6);
echo ":";
echo fseek($input, -5, SEEK_END);
echo ":";
echo stream_get_contents($input);
fclose($input);
"#,
    )
    .unwrap();

    let execution = run_program_with_options(
        &program,
        RunOptions {
            request_body: Some("action=save&token=abc".to_string()),
            request_method: Some("POST".to_string()),
            content_type: Some("application/json".to_string()),
            ..RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "resource|PHP:Input:rb:php://input|action=:7:save&token=abc:eof|action:0:n=abc"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn uploaded_file_builtins_use_seeded_phpc_files_provenance() {
    let source_path = temp_stream_path("phpc-upload-source.txt");
    let destination_path = temp_stream_path("phpc-upload-destination.txt");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&destination_path);
    fs::write(&source_path, "payload").expect("upload source can be seeded");
    let source = format!(
        r#"<?php
$tmp = "{}";
$dest = "{}";
echo function_exists("is_uploaded_file") ? "exists" : "missing";
echo "|";
echo is_callable("move_uploaded_file") ? "callable" : "not-callable";
echo "|";
echo is_uploaded_file($tmp) ? "uploaded" : "plain";
echo "|";
echo move_uploaded_file($tmp, $dest) ? "moved" : "stayed";
echo "|";
echo is_uploaded_file($tmp) ? "old-upload" : "old-clear";
echo "|";
echo file_exists($tmp) ? "old-exists" : "old-missing";
echo "|";
echo file_get_contents($dest);
"#,
        source_path.display(),
        destination_path.display()
    );
    let upload_files = format!(
        "async-upload[tmp_name]={}&async-upload[error]=0&async-upload[size]=7",
        form_encode_path(&source_path)
    );
    let program = parse(&source).unwrap();

    let execution = run_program_with_options(
        &program,
        RunOptions {
            upload_files: Some(upload_files),
            ..RunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "exists|callable|uploaded|moved|old-clear|old-missing|payload"
    );
    assert_eq!(execution.exit_code, 0);
    assert!(
        !source_path.exists(),
        "move_uploaded_file removes the source"
    );
    assert_eq!(
        fs::read_to_string(&destination_path).expect("destination is readable"),
        "payload"
    );
    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(destination_path);
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
fn stream_context_resources_store_options_and_are_accepted_by_bounded_stream_calls() {
    let path = temp_stream_path("phpc-stream-context-resource.txt");
    fs::write(&path, "context-file").expect("temporary stream context file can be written");
    let source = format!(
        r#"<?php
$context = stream_context_create(array(
    "http" => array("method" => "POST", "header" => "X-Test: one"),
    "ssl" => array("verify_peer" => false),
));
$options = stream_context_get_options($context);
echo gettype($context);
echo "|";
echo $options["http"]["method"];
echo ":";
echo $options["http"]["header"];
echo ":";
echo $options["ssl"]["verify_peer"] ? "verify" : "skip";
echo "|";
echo file_get_contents("{}", false, $context);
echo "|";
echo file_get_contents("{}", false, null);
echo "|";
$stream = fopen("{}", "r", false, $context);
$meta = stream_get_meta_data($stream);
echo $meta["wrapper_type"];
echo ":";
echo stream_get_contents($stream);
fclose($stream);
"#,
        path.display(),
        path.display(),
        path.display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "resource|POST:X-Test: one:skip|context-file|context-file|plainfile:context-file"
    );
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_file(path);
}

#[test]
fn stream_context_default_and_set_option_persist_bounded_options() {
    let path = temp_stream_path("phpc-stream-context-default.txt");
    fs::write(&path, "default-context").expect("temporary stream context file can be written");
    let source = format!(
        r#"<?php
$default = stream_context_get_default(array(
    "http" => array("method" => "GET"),
));
stream_context_set_option($default, "http", "header", "X-WP: one");
stream_context_set_option($default, array(
    "ssl" => array("verify_peer" => false),
    "http" => array("method" => "POST"),
));
$again = stream_context_get_default();
echo $again === $default ? "same" : "different";
$null_default = stream_context_get_default(null);
echo ":";
echo $null_default === $default ? "null-same" : "null-different";
$options = stream_context_get_options($again);
echo "|";
echo $options["http"]["method"];
echo ":";
echo $options["http"]["header"];
echo ":";
echo $options["ssl"]["verify_peer"] ? "verify" : "skip";
$replacement = stream_context_set_default(array(
    "http" => array("timeout" => 7),
));
echo "|";
echo $replacement === $default ? "same-default" : "new-default";
$default_options = stream_context_get_options(stream_context_get_default());
echo ":";
echo $default_options["http"]["timeout"];
echo "|";
echo file_get_contents("{}", false, $replacement);
"#,
        path.display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "same:null-same|POST:X-WP: one:skip|same-default:7|default-context"
    );
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_file(path);
}

#[test]
fn local_directory_handle_builtins_iterate_rewind_and_close_entries() {
    let root = temp_stream_path("phpc-directory-handle-root");
    let nested = root.join("nested");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&nested).expect("temporary directory fixture can be created");
    fs::write(root.join("alpha.txt"), "alpha").expect("alpha fixture file can be written");
    fs::write(root.join("beta.inc"), "<?php").expect("beta fixture file can be written");

    let source = format!(
        r#"<?php
$dir = opendir("{}");
echo gettype($dir);
echo "|";
echo readdir($dir);
echo ":";
echo readdir($dir);
$entries = array();
while (($entry = readdir($dir)) !== false) {{
    if ($entry !== "." && $entry !== "..") {{
        $entries[] = $entry;
    }}
}}
echo "|";
echo in_array("alpha.txt", $entries, true) ? "alpha" : "missing-alpha";
echo ":";
echo in_array("beta.inc", $entries, true) ? "beta" : "missing-beta";
echo ":";
echo in_array("nested", $entries, true) ? "nested" : "missing-nested";
echo ":";
echo count($entries);
rewinddir($dir);
echo "|";
echo readdir($dir);
closedir($dir);
echo "|";
echo opendir("{}") === false ? "missing-false" : "missing-open";
"#,
        root.display(),
        root.join("missing").display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "resource|.:..|alpha:beta:nested:3|.|missing-false"
    );
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stream_resource_builtins_reject_forms_outside_current_subset() {
    let wrapper = run_source("<?php\nfopen('http://example.test', 'r');\n").unwrap_err();
    assert_eq!(wrapper.phase, Phase::Runtime);
    assert_eq!(wrapper.line, 2);
    assert_eq!(wrapper.column, 1);
    assert_eq!(
        wrapper.message,
        "unsupported call fopen(): only php://memory, php://temp, php://input, and local file paths are supported in the current stream subset"
    );

    let bad_mode = run_source("<?php\nfopen('php://memory', 'x');\n").unwrap_err();
    assert_eq!(bad_mode.phase, Phase::Runtime);
    assert_eq!(bad_mode.line, 2);
    assert_eq!(bad_mode.column, 1);
    assert_eq!(
        bad_mode.message,
        "unsupported call fopen(): mode \"x\" is not supported in the current stream subset"
    );

    let bad_context =
        run_source("<?php\nfile_get_contents('php://input', false, 'ctx');\n").unwrap_err();
    assert_eq!(bad_context.phase, Phase::Runtime);
    assert_eq!(bad_context.line, 2);
    assert_eq!(bad_context.column, 1);
    assert_eq!(
        bad_context.message,
        "unsupported call file_get_contents(): context argument must be stream context resource in the current subset, got string"
    );

    let bad_context_options = run_source("<?php\nstream_context_create('ctx');\n").unwrap_err();
    assert_eq!(bad_context_options.phase, Phase::Runtime);
    assert_eq!(bad_context_options.line, 2);
    assert_eq!(bad_context_options.column, 1);
    assert_eq!(
        bad_context_options.message,
        "unsupported call stream_context_create(): options argument must be array or null in the current subset, got string"
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

    let bad_directory_wrapper = run_source("<?php\nopendir('php://memory');\n").unwrap_err();
    assert_eq!(bad_directory_wrapper.phase, Phase::Runtime);
    assert_eq!(bad_directory_wrapper.line, 2);
    assert_eq!(bad_directory_wrapper.column, 1);
    assert_eq!(
        bad_directory_wrapper.message,
        "unsupported call opendir(): stream wrappers are not supported in the current directory-handle subset"
    );

    let bad_readdir = run_source("<?php\nreaddir('not-resource');\n").unwrap_err();
    assert_eq!(bad_readdir.phase, Phase::Runtime);
    assert_eq!(bad_readdir.line, 2);
    assert_eq!(bad_readdir.column, 1);
    assert_eq!(
        bad_readdir.message,
        "unsupported call readdir(): directory argument must be resource in the current subset, got string"
    );

    let bad_upload_path = run_source("<?php\nis_uploaded_file(42);\n").unwrap_err();
    assert_eq!(bad_upload_path.phase, Phase::Runtime);
    assert_eq!(bad_upload_path.line, 2);
    assert_eq!(bad_upload_path.column, 1);
    assert_eq!(
        bad_upload_path.message,
        "unsupported call is_uploaded_file(): path argument must be string in the current subset, got int"
    );

    let bad_upload_wrapper =
        run_source("<?php\nmove_uploaded_file('php://input', '/tmp/file');\n").unwrap_err();
    assert_eq!(bad_upload_wrapper.phase, Phase::Runtime);
    assert_eq!(bad_upload_wrapper.line, 2);
    assert_eq!(bad_upload_wrapper.column, 1);
    assert_eq!(
        bad_upload_wrapper.message,
        "unsupported call move_uploaded_file(): stream wrappers are not supported in the current subset"
    );
}

#[test]
fn emit_ir_folds_stream_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("fopen") ? "1" : "0";
echo function_exists("stream_context_create") ? "1" : "0";
echo is_callable("stream_context_get_options") ? "1" : "0";
echo function_exists("stream_context_get_default") ? "1" : "0";
echo is_callable("stream_context_set_default") ? "1" : "0";
echo function_exists("stream_context_set_option") ? "1" : "0";
echo is_callable("stream_get_contents") ? "1" : "0";
echo defined("SEEK_END") ? "1" : "0";
echo function_exists("fstat") ? "1" : "0";
echo is_callable("stream_get_meta_data") ? "1" : "0";
echo function_exists("opendir") ? "1" : "0";
echo is_callable("readdir") ? "1" : "0";
echo function_exists("is_uploaded_file") ? "1" : "0";
echo is_callable("move_uploaded_file") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 14, "{ir}");

    let error = emit_ir_source("<?php\nis_uploaded_file('/tmp/phpc-upload');\n").unwrap_err();
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

fn form_encode_path(path: &PathBuf) -> String {
    path.to_string_lossy()
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                vec![byte as char]
            }
            b' ' => vec!['+'],
            other => format!("%{other:02X}").chars().collect(),
        })
        .collect()
}

#[test]
fn emit_asm_rejects_stream_resources_before_backend_execution() {
    let error = emit_asm_source("<?php\nstream_get_contents($stream);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STREAM_RESOURCE_REJECTION);
}
