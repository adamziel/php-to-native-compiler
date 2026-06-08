use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const LLVM_OBJECT_INSTANTIATION_REJECTION: &str = "LLVM object-instantiation lowering rejects new expressions and constructor dispatch until native object allocation, object handles, constructor calls, visibility checks, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors exist; phpc run handles current bounded new behavior";

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
class NamedCallable {
    public function instance() {}
    public static function direct() {}
    protected static function hidden() {}
    public static function __callStatic($name, $args) {}
}

echo is_callable("local_name") ? "1" : "0";
echo is_callable("LOCAL_NAME") ? "1" : "0";
echo is_callable("strlen") ? "1" : "0";
echo is_callable("extension_loaded") ? "1" : "0";
echo is_callable("dirname") ? "1" : "0";
echo is_callable("spl_autoload_register") ? "1" : "0";
echo is_callable("assert") ? "1" : "0";
echo is_callable("NamedCallable::direct") ? "1" : "0";
echo is_callable("NamedCallable::instance") ? "1" : "0";
echo is_callable("NamedCallable::hidden") ? "1" : "0";
echo is_callable("NamedCallable::missing") ? "1" : "0";
echo is_callable("MissingCallable::direct") ? "1" : "0";
echo is_callable("missing") ? "1" : "0";
echo is_callable(42) ? "1" : "0";
$arrow = fn($value) => $value;
echo is_callable($arrow) ? "1" : "0";
echo "\n";
$call = "is_callable";
echo $call("local_name") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "111111110110000\n1");
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
fn is_callable_writes_current_callable_name_output_subset_and_invokes_array_callables() {
    let syntax_error = run_source("<?php\nvar_dump(is_callable(\"missing\", 1));\n").unwrap_err();

    assert_eq!(syntax_error.phase, Phase::Runtime);
    assert_eq!(syntax_error.line, 2);
    assert_eq!(syntax_error.column, 10);
    assert_eq!(
        syntax_error.message,
        "unsupported call is_callable(): syntax_only argument must be bool in the current subset, got int"
    );

    let output_target_error =
        run_source("<?php\nvar_dump(is_callable(\"missing\", true, null));\n").unwrap_err();

    assert_eq!(output_target_error.phase, Phase::Runtime);
    assert_eq!(output_target_error.line, 2);
    assert_eq!(output_target_error.column, 39);
    assert_eq!(
        output_target_error.message,
        "unsupported call is_callable(): callable_name output must be a direct variable in the current subset"
    );

    let metadata = run_source(
        r#"<?php
function local_name() {}
class Box {
    public function open() {}
    public static function named() {}
    public function __call($name, $args) {}
    public static function __callStatic($name, $args) {}
}
$box = new Box();
$cases = [
    "local_name",
    "missing",
    "Box::named",
    "Box::open",
    [$box, "open"],
    [$box, "missing"],
    ["Box", "named"],
    ["Box", "open"],
    ["Box", "missing"],
    [1 => "Box", 2 => "open"],
    42,
    null,
];
foreach ($cases as $case) {
    $name = "seed";
    echo is_callable($case, false, $name) ? "1" : "0", ":", $name, "\n";
}
$name = "seed";
echo is_callable(["Missing", "open"], true, $name) ? "1" : "0", ":", $name;
"#,
    )
    .unwrap();

    assert_eq!(
        metadata.stdout,
        "1:local_name\n0:missing\n1:Box::named\n0:Box::open\n1:Box::open\n1:Box::missing\n1:Box::named\n0:Box::open\n1:Box::missing\n0:Array\n0:42\n0:\n1:Missing::open"
    );
    assert_eq!(metadata.exit_code, 0);

    let execution = run_source(
        r#"<?php
class Box {
    public function open() { echo "ok"; }
}
$box = new Box();
$callable = [$box, "open"];
$callable();
"#,
    )
    .expect("array callable variable invocation should execute");

    assert_eq!(execution.stdout, "ok");
    assert_eq!(execution.exit_code, 0);
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
echo function_exists("extension_loaded") ? "1" : "0";
echo function_exists("basename") ? "1" : "0";
echo function_exists("dirname") ? "1" : "0";
echo function_exists("spl_autoload_register") ? "1" : "0";
echo function_exists("assert") ? "1" : "0";
echo function_exists("missing") ? "1" : "0";
echo function_exists("not valid") ? "1" : "0";
echo "\n";
$call = "function_exists";
echo $call("local_name") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11111111100\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_exists_checks_namespaced_runtime_function_table_entries() {
    let execution = run_source(
        r#"<?php
namespace App\Demo;
function namespaced_name() {
    return "ok";
}

echo function_exists("App\\Demo\\namespaced_name") ? "1" : "0";
echo function_exists("APP\\DEMO\\NAMESPACED_NAME") ? "1" : "0";
echo function_exists("namespaced_name") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "110");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn extension_loaded_uses_current_compatibility_registry() {
    let execution = run_source(
        r#"<?php
echo extension_loaded("mbstring") ? "1" : "0";
echo extension_loaded("MBSTRING") ? "1" : "0";
echo extension_loaded("json") ? "1" : "0";
echo extension_loaded("HASH") ? "1" : "0";
echo extension_loaded("pdo") ? "1" : "0";
echo extension_loaded("pdo_mysql") ? "1" : "0";
echo "\n";
$call = "extension_loaded";
echo $call("simplexml") ? "1" : "0";
echo $call("hash") ? "1" : "0";
echo $call("pdo_mysql") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "001111\n011");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn extension_loaded_rejects_non_string_names_for_now() {
    let error = run_source("<?php\nvar_dump(extension_loaded(42));\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "unsupported call extension_loaded(): extension name argument must be string in the current subset, got int"
    );
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
        "<?php\necho is_int(1, 2) ? 1 : 0;\n",
        "<?php\necho is_callable(\"strlen\", 1) ? 1 : 0;\n",
        "<?php\necho is_callable(\"strlen\", true, $name) ? 1 : 0;\n",
        "<?php\necho function_exists(42) ? 1 : 0;\n",
        "<?php\necho function_exists(\"strlen\", true) ? 1 : 0;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }

    for source in [
        "<?php\n$call = \"is_array\";\necho $call([]) ? 1 : 0;\n",
        "<?php\n$call = \"is_callable\";\necho $call(\"strlen\") ? 1 : 0;\n",
        "<?php\n$call = \"function_exists\";\necho $call(\"strlen\") ? 1 : 0;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_DYNAMIC_FUNCTION_CALL_REJECTION);
    }

    let error = emit_ir_source("<?php\necho is_array([]) ? 1 : 0;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);

    let error = emit_ir_source("<?php\necho gettype(new Box()) ? 1 : 0;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
}

#[test]
fn emit_ir_folds_direct_function_exists_string_names() {
    let ir = emit_ir_source(
        r#"<?php
$known = "assert";
$missing = "missing_native_function";

echo function_exists("strlen") ? "1" : "0";
echo function_exists("STRLEN") ? "1" : "0";
echo function_exists("function_exists") ? "1" : "0";
echo function_exists("extension_loaded") ? "1" : "0";
echo function_exists("basename") ? "1" : "0";
echo function_exists("dirname") ? "1" : "0";
echo function_exists("spl_autoload_register") ? "1" : "0";
echo function_exists("assert") ? "1" : "0";
echo function_exists("ASSERT") ? "1" : "0";
echo function_exists("missing_native_function") ? "1" : "0";
echo function_exists($known) ? "1" : "0";
echo function_exists($missing) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 10, "{ir}");
    assert_eq!(ir.matches("c\"0\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
}

#[test]
fn emit_ir_folds_direct_extension_loaded_string_names() {
    let ir = emit_ir_source(
        r#"<?php
$name = "mbstring";

echo extension_loaded("mbstring") ? "1" : "0";
echo extension_loaded("MBSTRING") ? "1" : "0";
echo extension_loaded("json") ? "1" : "0";
echo extension_loaded("HASH") ? "1" : "0";
echo extension_loaded("pdo") ? "1" : "0";
echo extension_loaded("pdo_mysql") ? "1" : "0";
echo extension_loaded($name) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert_eq!(ir.matches("c\"0\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("extension_loaded"), "{ir}");
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
echo is_callable("extension_loaded") ? "1" : "0";
echo is_callable("basename") ? "1" : "0";
echo is_callable("dirname") ? "1" : "0";
echo is_callable("spl_autoload_register") ? "1" : "0";
echo is_callable("assert") ? "1" : "0";
echo is_callable("ASSERT") ? "1" : "0";
echo is_callable("missing_native_function") ? "1" : "0";
echo is_callable("missing_native_function", true) ? "1" : "0";
echo is_callable("assert", false) ? "1" : "0";
echo is_callable("strlen", false) ? "1" : "0";
echo is_callable($known) ? "1" : "0";
echo is_callable($missing) ? "1" : "0";
echo is_callable($missing, $syntax) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 13, "{ir}");
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
