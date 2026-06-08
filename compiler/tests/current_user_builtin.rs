use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source_with_source_file};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn get_current_user_returns_string_and_metadata() {
    let source = r#"<?php
$name = get_current_user();
echo is_string($name) ? "string" : "bad";
echo "|";
foreach (["get_current_user"] as $call) {
    echo function_exists($call) ? "1" : "0";
    echo is_callable($call) ? "1" : "0";
}
echo "|";
$reflection = new ReflectionFunction("get_current_user");
echo $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters();
echo ":", $reflection->getReturnType()->getName();
echo ":", is_string($reflection->invoke()) ? "invoke" : "bad";
"#;
    let path = temp_source_path("current-user");
    fs::write(&path, source).expect("temporary current-user source is written");
    let execution = run_source_with_source_file(source, path.display().to_string()).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(execution.stdout, "string|11|0/0:string:invoke");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_get_current_user_metadata_but_rejects_direct_call() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("get_current_user") ? "1" : "0";
echo is_callable("get_current_user") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho get_current_user();\n").unwrap_err();
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
