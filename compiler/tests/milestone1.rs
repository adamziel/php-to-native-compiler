use std::path::Path;
use std::process::Command;

use php_compiler::test_runner::run_fixture_dir;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

#[test]
fn milestone1_fixtures_pass() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
    let summary = run_fixture_dir(&fixture_dir).unwrap();
    assert_eq!(summary.failed, 0, "{:#?}", summary.failures);
    assert!(summary.passed >= 13);
}

#[test]
fn run_executes_function_and_loop() {
    let source = r#"<?php
function twice($x) {
    return $x * 2;
}
$i = 1;
while ($i < 4) {
    echo twice($i), ",";
    $i = $i + 1;
}
"#;
    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "2,4,6,");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_for_integer_arithmetic() {
    let ir = emit_ir_source("<?php\n$x = 1 + 2;\necho $x;\n").unwrap();
    assert!(ir.contains("add i64 1, 2"), "{ir}");
    assert!(ir.contains("@printf"), "{ir}");
    assert!(ir.contains("define i32 @main()"), "{ir}");
}

#[test]
fn emit_ir_rejects_unsupported_control_flow() {
    let error = emit_ir_source("<?php\nif (1) { echo 1; }\n").unwrap_err();
    assert_eq!(error.phase, php_compiler::error::Phase::Codegen);
    assert!(error.message.contains("if/else"));
}

#[test]
fn emit_asm_through_available_native_toolchain() {
    let has_backend = ["clang", "llc", "cc"]
        .iter()
        .any(|command| Command::new(command).arg("--version").output().is_ok());
    if !has_backend {
        return;
    }

    let asm = emit_asm_source("<?php\necho 1 + 2;\n").unwrap();
    assert!(asm.contains("main"), "{asm}");
}
