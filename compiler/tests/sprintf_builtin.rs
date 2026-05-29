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
echo sprintf('%.4d|%b|%c|%u|%o|%x|%X|%.2e', -42, 5, 65, 9, 9, 255, 255, 1000), "\n";
echo sprintf('%e|%u|%o', +3e3, +2345432, +0567), "\n";
echo sprintf('%g|%.3G|%*.*f|%.*s|%d', 1.234567, 12345.0, 8, 2, 1.25, 3, "abcdef", "world"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Hello Ada\ntwo:one\n% done done\n00007|12.00|    x|y    |7|z\n-0042|101|A|9|11|ff|FF|1.00e+3\n3.000000e+3|2345432|567\n1.23457|1.23E+4|    1.25|abc|0\n"
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
$call = "vprintf";
echo "|", function_exists($call) ? "yes" : "no";
echo "|";
$length = $call('%2$s/%1$s', ["left", "right"]);
echo "|", $length;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|wp-php|yes|callable|SELECT option_value FROM wp_options WHERE option_name = 'rewrite_rules' LIMIT 1|yes|right/left|10"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn vprintf_outputs_formatted_string_and_returns_byte_length() {
    let execution = run_source(
        r#"<?php
$length = vprintf("%s:%04d:%x", ["id", 7, 255]);
echo "\n", $length, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "id:0007:ff\n10\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_encode_covers_current_formatter_descriptor_values() {
    let execution = run_source(
        r#"<?php
echo json_encode(null), "|", json_encode(false), "|", json_encode(true), "\n";
echo json_encode([1, "x"]), "|", json_encode(["color" => "red"]), "|", json_encode(new stdClass), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "null|false|true\n[1,\"x\"]|{\"color\":\"red\"}|{}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_rejects_format_forms_outside_current_subset() {
    let placeholder = runtime_error(
        r#"<?php
echo sprintf("%a", 4);
"#,
    );
    assert_eq!(placeholder.line, 2);
    assert_eq!(placeholder.column, 6);
    assert_eq!(
        placeholder.message,
        "unsupported call sprintf(): unsupported format placeholder %a in the current subset"
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

    let vsprintf_star = runtime_error(
        r#"<?php
echo vsprintf("%*2f", [1, 2]);
"#,
    );
    assert_eq!(vsprintf_star.line, 2);
    assert_eq!(vsprintf_star.column, 6);
    assert_eq!(
        vsprintf_star.message,
        "unsupported call vsprintf(): unsupported format placeholder %* in the current subset"
    );

    let vprintf_args = runtime_error(
        r#"<?php
echo vprintf("%s", "not-array");
"#,
    );
    assert_eq!(vprintf_args.line, 2);
    assert_eq!(vprintf_args.column, 6);
    assert_eq!(
        vprintf_args.message,
        "unsupported call vprintf(): values argument must be array in the current subset, got string"
    );
}

#[test]
fn emit_ir_folds_sprintf_metadata_but_rejects_runtime_formatting_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("sprintf") ? "1" : "0";
echo is_callable("vsprintf") ? "1" : "0";
echo function_exists("vprintf") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
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

    let error = emit_ir_source(
        r#"<?php
echo vprintf("Hello %s", ["Ada"]);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
