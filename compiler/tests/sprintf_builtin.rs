use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn sprintf_executes_current_string_placeholder_subset() {
    let execution = run_source(
        r#"<?php
echo sprintf("Hello %s", "Ada"), "\n";
echo sprintf('%2$s:%1$s', "one", "two"), "\n";
echo sprintf('%% %s %1$s', "done"), "\n";
echo sprintf('%05d|%.2F|%5s|%-5s|%1$s|%s', "7", 12, "x", "y", "z"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Hello Ada\ntwo:one\n% done done\n00007|12.00|    x|y    |7|z\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_and_vsprintf_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "sprintf";
echo function_exists($call) ? "yes" : "no";
echo "|", $call('%1$s-%2$s', "wp", "php");
echo "|";
$call = "vsprintf";
echo function_exists($call) ? "yes" : "no";
echo "|", is_callable($call) ? "callable" : "missing";
echo "|", $call("SELECT option_value FROM wp_options WHERE option_name = '%s' LIMIT %d", ["rewrite_rules", 1]);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|wp-php|yes|callable|SELECT option_value FROM wp_options WHERE option_name = 'rewrite_rules' LIMIT 1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_rejects_format_forms_outside_current_subset() {
    let format = runtime_error(
        r#"<?php
echo sprintf(42, "x");
"#,
    );
    assert_eq!(format.line, 2);
    assert_eq!(format.column, 6);
    assert_eq!(
        format.message,
        "unsupported call sprintf(): format argument must be string in the current subset, got int"
    );

    let placeholder = runtime_error(
        r#"<?php
echo sprintf("%x", 4);
"#,
    );
    assert_eq!(placeholder.line, 2);
    assert_eq!(placeholder.column, 6);
    assert_eq!(
        placeholder.message,
        "unsupported call sprintf(): unsupported format placeholder %x in the current subset"
    );

    let missing = runtime_error(
        r#"<?php
echo sprintf('%2$s', "only-one");
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "unsupported call sprintf(): missing argument for placeholder 2"
    );

    let vsprintf_args = runtime_error(
        r#"<?php
echo vsprintf("%s", "not-array");
"#,
    );
    assert_eq!(vsprintf_args.line, 2);
    assert_eq!(vsprintf_args.column, 6);
    assert_eq!(
        vsprintf_args.message,
        "unsupported call vsprintf(): values argument must be array in the current subset, got string"
    );

    let vsprintf_numeric = runtime_error(
        r#"<?php
echo vsprintf("%d", ["abc"]);
"#,
    );
    assert_eq!(vsprintf_numeric.line, 2);
    assert_eq!(vsprintf_numeric.column, 6);
    assert_eq!(
        vsprintf_numeric.message,
        "unsupported call vsprintf(): numeric placeholders require numeric scalar arguments in the current subset"
    );
}

#[test]
fn emit_ir_folds_sprintf_metadata_but_rejects_runtime_formatting_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("sprintf") ? "1" : "0";
echo is_callable("vsprintf") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source(
        r#"<?php
echo sprintf("Hello %s", "Ada");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
echo vsprintf("Hello %s", ["Ada"]);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
