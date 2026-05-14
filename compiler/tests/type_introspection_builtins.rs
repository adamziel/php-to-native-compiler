use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";

#[test]
fn gettype_reports_php_legacy_type_names_for_current_values() {
    let execution = run_source(
        r#"<?php
class Box {}

$box = new Box();
$values = [null, false, 7, 3.5, "x", ["nested"], $box];
foreach ($values as $value) {
    echo gettype($value), "\n";
}
$call = "gettype";
echo $call(true), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "NULL\nboolean\ninteger\ndouble\nstring\narray\nobject\nboolean\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn type_predicates_cover_current_value_model_and_aliases() {
    let execution = run_source(
        r#"<?php
class Box {}

$box = new Box();
echo is_null(null) ? "1" : "0", is_null(0) ? "1" : "0", "\n";
echo is_bool(false) ? "1" : "0", is_bool(0) ? "1" : "0", "\n";
echo is_int(7) ? "1" : "0", is_integer(7) ? "1" : "0", is_long(7) ? "1" : "0", is_int("7") ? "1" : "0", "\n";
echo is_float(3.5) ? "1" : "0", is_double(3.5) ? "1" : "0", is_float(3) ? "1" : "0", "\n";
echo is_string("x") ? "1" : "0", is_string(1) ? "1" : "0", "\n";
echo is_array(["x"]) ? "1" : "0", is_array($box) ? "1" : "0", "\n";
echo is_scalar(false) ? "1" : "0", is_scalar(1) ? "1" : "0", is_scalar(1.5) ? "1" : "0", is_scalar("x") ? "1" : "0", is_scalar(null) ? "1" : "0", is_scalar(["x"]) ? "1" : "0", is_scalar($box) ? "1" : "0", "\n";
$call = "is_array";
echo $call(["dynamic"]) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "10\n10\n1110\n110\n10\n10\n1111000\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_callable_checks_current_string_function_name_subset() {
    let execution = run_source(
        r#"<?php
function local_name() {
    return "ok";
}

echo is_callable("local_name") ? "1" : "0";
echo is_callable("LOCAL_NAME") ? "1" : "0";
echo is_callable("strlen") ? "1" : "0";
echo is_callable("missing") ? "1" : "0";
echo is_callable(42) ? "1" : "0";
echo "\n";
$call = "is_callable";
echo $call("local_name") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11100\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_callable_supports_current_string_syntax_only_subset() {
    let execution = run_source(
        r#"<?php
function local_name() {}

echo is_callable("missing", true) ? "1" : "0";
echo is_callable("not valid", true) ? "1" : "0";
echo is_callable("local_name", false) ? "1" : "0";
echo is_callable("missing", false) ? "1" : "0";
echo is_callable(42, true) ? "1" : "0";
echo is_callable(null, true) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "111000");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_callable_supports_current_array_syntax_only_subset() {
    let execution = run_source(
        r#"<?php
class Box {
    public function open() {
        return "ok";
    }
}

$box = new Box();
echo is_callable(["Box", "open"], true) ? "1" : "0";
echo is_callable([$box, "open"], true) ? "1" : "0";
echo is_callable(["Missing", "open"], true) ? "1" : "0";
echo is_callable(["Box", "not valid"], true) ? "1" : "0";
echo is_callable([1 => "Box", 2 => "open"], true) ? "1" : "0";
echo is_callable(["class" => "Box", "method" => "open"], true) ? "1" : "0";
echo is_callable(["Box"], true) ? "1" : "0";
echo is_callable(["Box", 42], true) ? "1" : "0";
echo is_callable([42, "open"], true) ? "1" : "0";
echo is_callable($box, true) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1111000000");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_callable_resolves_current_array_callable_metadata_subset() {
    let execution = run_source(
        r#"<?php
class Box {
    public function open() {
        return "ok";
    }
    protected function seal() {}
    private static function cache() {}
    public static function named() {
        return "ok";
    }
}

$box = new Box();
echo is_callable(["Box", "open"]) ? "1" : "0";
echo is_callable(["BOX", "OPEN"]) ? "1" : "0";
echo is_callable([$box, "open"]) ? "1" : "0";
echo is_callable([$box, "seal"]) ? "1" : "0";
echo is_callable([$box, "cache"]) ? "1" : "0";
echo is_callable(["Box", "cache"]) ? "1" : "0";
echo is_callable(["Box", "named"]) ? "1" : "0";
echo is_callable([$box, "named"]) ? "1" : "0";
echo is_callable(["Missing", "open"]) ? "1" : "0";
echo is_callable(["Box", "missing"]) ? "1" : "0";
echo is_callable([1 => "Box", 2 => "open"]) ? "1" : "0";
echo is_callable(["Box", 42]) ? "1" : "0";
echo is_callable([42, "open"]) ? "1" : "0";
echo is_callable(["Box", "named"], false) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "00100011000001");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn is_callable_rejects_unsupported_output_arguments_and_invocation_for_now() {
    let syntax_error = run_source("<?php\nvar_dump(is_callable(\"missing\", 1));\n").unwrap_err();

    assert_eq!(syntax_error.phase, Phase::Runtime);
    assert_eq!(syntax_error.line, 2);
    assert_eq!(syntax_error.column, 10);
    assert_eq!(
        syntax_error.message,
        "unsupported call is_callable(): syntax_only argument must be bool in the current subset, got int"
    );

    let output_error =
        run_source("<?php\nvar_dump(is_callable(\"missing\", true, null));\n").unwrap_err();

    assert_eq!(output_error.phase, Phase::Runtime);
    assert_eq!(output_error.line, 2);
    assert_eq!(output_error.column, 10);
    assert_eq!(
        output_error.message,
        "arity mismatch for is_callable(): expected 1 to 2 argument(s), got 3"
    );

    let invocation_error = run_source(
        r#"<?php
class Box {
    public function open() {}
}
$box = new Box();
$callable = [$box, "open"];
$callable();
"#,
    )
    .unwrap_err();

    assert_eq!(invocation_error.phase, Phase::Runtime);
    assert_eq!(invocation_error.line, 7);
    assert_eq!(invocation_error.column, 1);
    assert_eq!(
        invocation_error.message,
        "unsupported call dynamic function call: callable expression must evaluate to string, got array"
    );
}

#[test]
fn function_exists_checks_current_runtime_function_table() {
    let execution = run_source(
        r#"<?php
function local_name() {
    return "ok";
}

echo function_exists("local_name") ? "1" : "0";
echo function_exists("LOCAL_NAME") ? "1" : "0";
echo function_exists("strlen") ? "1" : "0";
echo function_exists("function_exists") ? "1" : "0";
echo function_exists("missing") ? "1" : "0";
echo function_exists("not valid") ? "1" : "0";
echo "\n";
$call = "function_exists";
echo $call("local_name") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "111100\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_exists_rejects_non_string_names_for_now() {
    let error = run_source("<?php\nvar_dump(function_exists(42));\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "unsupported call function_exists(): function name argument must be string in the current subset, got int"
    );
}

#[test]
fn emit_ir_folds_direct_scalar_null_type_introspection_calls() {
    let ir = emit_ir_source(
        r#"<?php
$null = null;
$bool = false;
$int = 7;
$float = 3.5;
$string = "x";

echo gettype($null), "\n";
echo gettype($bool), "\n";
echo gettype($int), "\n";
echo gettype($float), "\n";
echo gettype($string), "\n";
echo is_null($null) ? "1" : "0", is_null($int) ? "1" : "0", "\n";
echo is_bool($bool) ? "1" : "0", is_bool($int) ? "1" : "0", "\n";
echo is_int($int) ? "1" : "0", is_integer($int) ? "1" : "0", is_long($int) ? "1" : "0", is_int($string) ? "1" : "0", "\n";
echo is_float($float) ? "1" : "0", is_double($float) ? "1" : "0", is_float($int) ? "1" : "0", "\n";
echo is_string($string) ? "1" : "0", is_string($int) ? "1" : "0", "\n";
echo is_array($string) ? "1" : "0", "\n";
echo is_scalar($bool) ? "1" : "0", is_scalar($int) ? "1" : "0", is_scalar($float) ? "1" : "0", is_scalar($string) ? "1" : "0", is_scalar($null) ? "1" : "0";
"#,
    )
    .unwrap();

    for expected in [
        "c\"NULL\\00\"",
        "c\"boolean\\00\"",
        "c\"integer\\00\"",
        "c\"double\\00\"",
        "c\"string\\00\"",
        "c\"1\\00\"",
        "c\"0\\00\"",
    ] {
        assert!(ir.contains(expected), "{ir}");
    }
    assert!(!ir.contains("@strcmp"), "{ir}");
}

#[test]
fn emit_ir_keeps_unsupported_type_introspection_boundaries_explicit() {
    for source in [
        "<?php\n$call = \"is_array\";\necho $call([]) ? 1 : 0;\n",
        "<?php\necho is_int(1, 2) ? 1 : 0;\n",
        "<?php\necho is_callable(\"strlen\", 1) ? 1 : 0;\n",
        "<?php\necho is_callable(\"strlen\", true, $name) ? 1 : 0;\n",
        "<?php\n$call = \"is_callable\";\necho $call(\"strlen\") ? 1 : 0;\n",
        "<?php\necho function_exists(42) ? 1 : 0;\n",
        "<?php\necho function_exists(\"strlen\", true) ? 1 : 0;\n",
        "<?php\n$call = \"function_exists\";\necho $call(\"strlen\") ? 1 : 0;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }

    let error = emit_ir_source("<?php\necho is_array([]) ? 1 : 0;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);

    let error = emit_ir_source("<?php\necho gettype(new Box()) ? 1 : 0;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_folds_direct_function_exists_string_names() {
    let ir = emit_ir_source(
        r#"<?php
$known = "strlen";
$missing = "missing_native_function";

echo function_exists("strlen") ? "1" : "0";
echo function_exists("STRLEN") ? "1" : "0";
echo function_exists("function_exists") ? "1" : "0";
echo function_exists("missing_native_function") ? "1" : "0";
echo function_exists($known) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert_eq!(ir.matches("c\"0\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
}

#[test]
fn emit_ir_folds_direct_is_callable_string_names() {
    let ir = emit_ir_source(
        r#"<?php
$known = "strlen";
$missing = "missing_native_function";
$syntax = true;

echo is_callable("strlen") ? "1" : "0";
echo is_callable("STRLEN") ? "1" : "0";
echo is_callable("missing_native_function") ? "1" : "0";
echo is_callable("missing_native_function", true) ? "1" : "0";
echo is_callable("strlen", false) ? "1" : "0";
echo is_callable($known) ? "1" : "0";
echo is_callable($missing) ? "1" : "0";
echo is_callable($missing, $syntax) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 6, "{ir}");
    assert_eq!(ir.matches("c\"0\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}

#[test]
fn emit_ir_includes_array_change_key_case_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
$name = "array_change_key_case";
$missing = "missing_native_function";

echo function_exists("array_change_key_case") ? "1" : "0";
echo function_exists("ARRAY_CHANGE_KEY_CASE") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo is_callable("array_change_key_case") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 5, "{ir}");
    assert_eq!(ir.matches("c\"0\\00\"").count(), 1, "{ir}");
    assert!(!ir.contains("array_change_key_case"), "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}

#[test]
fn emit_ir_folds_direct_is_callable_non_string_scalars_to_false() {
    let ir = emit_ir_source(
        r#"<?php
$flag = true;
$syntax = true;

echo is_callable(null) ? "1" : "0";
echo is_callable(false) ? "1" : "0";
echo is_callable(42) ? "1" : "0";
echo is_callable(3.5) ? "1" : "0";
echo is_callable($flag) ? "1" : "0";
echo is_callable(42, true) ? "1" : "0";
echo is_callable(false, $syntax) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"0\\00\"").count(), 7, "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
