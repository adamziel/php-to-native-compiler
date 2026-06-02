#![cfg(unix)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source_with_source_file};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn posix_lookup_builtins_return_local_account_and_group_arrays() {
    let source = r#"<?php
$pw = posix_getpwuid(getmyuid());
$gr = posix_getgrgid(getmygid());
echo function_exists("posix_getpwuid") && is_callable("posix_getgrgid") ? "known" : "missing";
echo "|";
echo is_array($pw)
    && is_string($pw["name"])
    && is_string($pw["passwd"])
    && is_int($pw["uid"])
    && is_int($pw["gid"])
    && array_key_exists("gecos", $pw)
    && array_key_exists("dir", $pw)
    && array_key_exists("shell", $pw)
    ? "pw-array"
    : "bad-pw";
echo "|";
echo is_array($gr)
    && is_string($gr["name"])
    && is_string($gr["passwd"])
    && is_array($gr["members"])
    && is_int($gr["gid"])
    ? "gr-array"
    : "bad-gr";
echo "|";
$rf = new ReflectionFunction("posix_getpwuid");
echo $rf->getNumberOfRequiredParameters(), "/", $rf->getNumberOfParameters(), "/";
echo $rf->getParameters()[0]->getName(), "/", $rf->getExtensionName(), "/";
echo $rf->hasReturnType() ? "return" : "no-return";
echo "|";
$viaInvoke = $rf->invoke(getmyuid());
echo is_array($viaInvoke) && is_int($viaInvoke["uid"]) ? "invoke-array" : "bad-invoke";
echo "|";
var_dump(posix_getpwuid(-99));
var_dump(posix_getgrgid(-999));
"#;
    let path = temp_source_path("posix-lookup");
    fs::write(&path, source).expect("temporary POSIX lookup source is written");

    let execution = run_source_with_source_file(source, path.display().to_string()).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "known|pw-array|gr-array|1/1/user_id/posix/return|invoke-array|bool(false)\nbool(false)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_posix_lookup_names_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("posix_getpwuid") ? "1" : "0";
echo is_callable("posix_getgrgid") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho posix_getpwuid(0);\n").unwrap_err();
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
