use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn dirname_executes_current_unix_path_subset() {
    let execution = run_source(
        r#"<?php
echo dirname("/tmp/wordpress/wp-includes/sodium_compat/autoload.php"), "\n";
echo dirname("/tmp/wordpress/wp-includes/sodium_compat/"), "\n";
echo "[", dirname("autoload.php"), "]\n";
echo "[", dirname(""), "]\n";
echo dirname("/a/b/c.php", 2), "\n";
$call = "dirname";
echo $call("/a/b//c.php");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "/tmp/wordpress/wp-includes/sodium_compat\n/tmp/wordpress/wp-includes\n[.]\n[]\n/a\n/a/b"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dirname_reports_current_argument_boundaries() {
    let non_string_path = run_source("<?php\necho dirname(42);\n").unwrap_err();
    assert_eq!(non_string_path.phase, Phase::Runtime);
    assert_eq!(non_string_path.line, 2);
    assert_eq!(non_string_path.column, 6);
    assert_eq!(
        non_string_path.message,
        "unsupported call dirname(): path argument must be string in the current subset, got int"
    );

    let non_positive_levels = run_source("<?php\necho dirname('/a', 0);\n").unwrap_err();
    assert_eq!(non_positive_levels.phase, Phase::Runtime);
    assert_eq!(non_positive_levels.line, 2);
    assert_eq!(non_positive_levels.column, 6);
    assert_eq!(
        non_positive_levels.message,
        "unsupported call dirname(): levels argument must be greater than or equal to 1 in the current subset"
    );

    let non_int_levels = run_source("<?php\necho dirname('/a', '2');\n").unwrap_err();
    assert_eq!(non_int_levels.phase, Phase::Runtime);
    assert_eq!(non_int_levels.line, 2);
    assert_eq!(non_int_levels.column, 6);
    assert_eq!(
        non_int_levels.message,
        "unsupported call dirname(): levels argument must be int in the current subset, got string"
    );
}

#[test]
fn emit_ir_rejects_direct_dirname_until_native_path_lowering_exists() {
    let error = emit_ir_source("<?php\necho dirname('/a/b.php');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
