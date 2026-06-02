use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source_with_source_file};

const LLVM_CLEARSTATCACHE_REJECTION: &str = "LLVM clearstatcache lowering rejects stat-cache mutation until native filesystem metadata caches, realpath cache state, per-path invalidation, include_path/open_basedir policy, stream wrappers, request-local filesystem state, references/COW, and exact native diagnostics exist; phpc run handles current bounded stat-cache clearstatcache behavior";

fn fixture_source_file() -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("compiler has a workspace root")
        .join("tests/fixtures/milestone1538/clearstatcache_metadata.php")
        .display()
        .to_string()
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source_with_source_file(source, fixture_source_file()).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn php_single_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

#[test]
fn clearstatcache_is_available_and_returns_null() {
    let execution = run_source_with_source_file(
        r#"<?php
echo function_exists("clearstatcache") ? "known" : "missing";
echo "|";
echo is_callable("clearstatcache") ? "callable" : "not-callable";
echo "|";
$call = "clearstatcache";
$first = clearstatcache();
$second = $call(true, __FILE__);
echo $first === null ? "first-null" : "first-value";
echo "|";
echo $second === null ? "second-null" : "second-value";
echo "|";
echo file_exists(__FILE__) ? "exists" : "missing";
echo "|";
echo is_int(filemtime(__FILE__)) ? "mtime-int" : "mtime-false";
echo "|";
echo filesize(__FILE__) > 0 ? "size-positive" : "size-empty";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "known|callable|first-null|second-null|exists|mtime-int|size-positive"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn local_writes_invalidate_bounded_filesize_stat_cache_for_link_aliases() {
    let path = std::env::temp_dir().join(format!(
        "phpc-stat-cache-{}-{}.txt",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos()
    ));
    let alias_path = path.with_extension("alias.txt");
    let path_string = path
        .to_str()
        .expect("temporary stat-cache fixture path is UTF-8");
    let alias_path_string = alias_path
        .to_str()
        .expect("temporary stat-cache alias path is UTF-8");
    let path_literal = php_single_quoted(path_string);
    let alias_path_literal = php_single_quoted(alias_path_string);
    let source = format!(
        r#"<?php
$path = '{path_literal}';
$alias = '{alias_path_literal}';
$h = fopen($path, "w");
fwrite($h, "abc");
fclose($h);
link($path, $alias);
$first = filesize($path);
$h = fopen($alias, "w");
fwrite($h, "abcdef");
fclose($h);
$cached = filesize($path);
clearstatcache(false, $path);
$cleared = filesize($path);
$h = fopen($alias, "w");
fwrite($h, "abcdefghi");
fclose($h);
$cached_again = filesize($path);
clearstatcache();
$cleared_all = filesize($path);
echo $first;
echo "|";
echo $cached;
echo "|";
echo $cleared;
echo "|";
echo $cached_again;
echo "|";
echo $cleared_all;
"#
    );

    let execution = run_source_with_source_file(&source, fixture_source_file()).unwrap();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(alias_path);

    assert_eq!(execution.stdout, "3|6|6|9|9");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn clearstatcache_rejects_forms_outside_current_subset() {
    let arity = runtime_error(
        r#"<?php
clearstatcache(false, __FILE__, "extra");
"#,
    );
    assert_eq!(arity.line, 2);
    assert_eq!(arity.column, 1);
    assert_eq!(
        arity.message,
        "arity mismatch for clearstatcache(): expected 0 to 2 argument(s), got 3"
    );

    let clear_realpath_type = runtime_error(
        r#"<?php
clearstatcache("yes");
"#,
    );
    assert_eq!(clear_realpath_type.line, 2);
    assert_eq!(clear_realpath_type.column, 1);
    assert_eq!(
        clear_realpath_type.message,
        "unsupported call clearstatcache(): clear_realpath_cache argument must be bool in the current subset, got string"
    );

    let filename_type = runtime_error(
        r#"<?php
clearstatcache(true, 42);
"#,
    );
    assert_eq!(filename_type.line, 2);
    assert_eq!(filename_type.column, 1);
    assert_eq!(
        filename_type.message,
        "unsupported call clearstatcache(): filename argument must be string in the current subset, got int"
    );
}

#[test]
fn emit_ir_rejects_clearstatcache_until_native_stat_cache_exists() {
    let error = emit_ir_source(
        r#"<?php
clearstatcache(true, "wp-config.php");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_CLEARSTATCACHE_REJECTION);
}

#[test]
fn emit_asm_rejects_clearstatcache_until_native_stat_cache_exists() {
    let error = emit_asm_source(
        r#"<?php
clearstatcache(true, "wp-config.php");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_CLEARSTATCACHE_REJECTION);
}
