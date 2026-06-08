use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source, run_source_with_source_file};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn script_metadata_builtins_return_main_file_stat_values_and_metadata() {
    let source = r#"<?php
$checks = array(
    "lastmod" => is_int(getlastmod()) && getlastmod() > 0,
    "inode" => is_int(getmyinode()) && getmyinode() > 0,
    "uid" => is_int(getmyuid()) && getmyuid() >= 0,
    "gid" => is_int(getmygid()) && getmygid() >= 0,
    "pid" => is_int(getmypid()) && getmypid() > 0,
);
foreach ($checks as $name => $ok) {
    echo $ok ? $name : "bad-$name";
    echo "|";
}
foreach (["getlastmod", "getmyinode", "getmyuid", "getmygid"] as $name) {
    echo function_exists($name) && is_callable($name) ? "1" : "0";
    $reflection = new ReflectionFunction($name);
    echo $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters(), "|";
}
"#;
    let path = temp_source_path("script-metadata");
    fs::write(&path, source).expect("temporary script metadata source is written");

    let execution = run_source_with_source_file(source, path.display().to_string()).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "lastmod|inode|uid|gid|pid|10/0|10/0|10/0|10/0|"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn script_metadata_builtins_return_false_without_main_source_file() {
    let execution = run_source(
        r#"<?php
var_dump(getlastmod());
var_dump(getmyinode());
var_dump(getmyuid());
var_dump(getmygid());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(false)\nbool(false)\nbool(false)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_script_metadata_names_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("getlastmod") ? "1" : "0";
echo is_callable("getmyinode") ? "1" : "0";
echo function_exists("getmyuid") ? "1" : "0";
echo is_callable("getmygid") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho getlastmod();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

fn temp_source_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "phpc-{label}-{}-{}.php",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos()
    ))
}
