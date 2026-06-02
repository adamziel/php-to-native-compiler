use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";

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
fn gettype_reports_closed_resource_legacy_name() {
    let execution = run_source(
        r#"<?php
$stream = fopen("php://memory", "w+");
echo gettype($stream), "\n";
fclose($stream);
echo gettype($stream), "\n";
echo gettype(STDIN), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "resource\nresource (closed)\nresource\n");
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
echo is_callable("extension_loaded") ? "1" : "0";
echo is_callable("dirname") ? "1" : "0";
echo is_callable("spl_autoload_register") ? "1" : "0";
echo is_callable("assert") ? "1" : "0";
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

    assert_eq!(execution.stdout, "1111111000\n1");
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
$cases = [
    null,
    0,
    123,
    -2.0,
    .567,
    false,
    [1, 2, 3],
    "strlen",
    "missing",
];
foreach ($cases as $case) {
    $name = "seed";
    echo is_callable($case, true, $name) ? "1" : "0", ":", $name, "\n";
}
$name = "seed";
echo is_callable("strlen", false, $name) ? "1" : "0", ":", $name, "\n";
$name = "seed";
echo is_callable("missing", false, $name) ? "1" : "0", ":", $name;
"#,
    )
    .unwrap();

    assert_eq!(
        metadata.stdout,
        "0:\n0:0\n0:123\n0:-2\n0:0.567\n0:\n0:Array\n1:strlen\n1:missing\n1:strlen\n0:missing"
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
fn metadata_predicates_use_php_type_errors_and_class_string_method_visibility() {
    let execution = run_source(
        r#"<?php
class BaseMetadata {
    public function pub() {}
    protected function prot() {}
    private function hidden() {}
    private static function staticHidden() {}
}
class ChildMetadata extends BaseMetadata {
    private function ownHidden() {}
}

set_error_handler(function($errno, $message) {
    echo "warning:", $errno, ":", $message, "\n";
});
$array = [];
echo "interpolated:$array\n";

foreach ([[], 1, 3.5, true, null] as $value) {
    try {
        property_exists($value, "pub");
    } catch (Throwable $e) {
        echo $e::class, ": ", $e->getMessage(), "\n";
    }
}

try {
    method_exists(false, "pub");
} catch (Throwable $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}

try {
    method_exists(new ChildMetadata(), []);
} catch (Throwable $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}

echo method_exists("ChildMetadata", "pub") ? "1" : "0";
echo method_exists("ChildMetadata", "prot") ? "1" : "0";
echo method_exists("ChildMetadata", "hidden") ? "1" : "0";
echo method_exists("ChildMetadata", "staticHidden") ? "1" : "0";
echo method_exists("ChildMetadata", "ownHidden") ? "1" : "0";
echo method_exists(new ChildMetadata(), "hidden") ? "1" : "0";
echo "\n";

foreach ([0, 1.5, [], null, false, "", "ChildMetadata"] as $value) {
    echo is_subclass_of($value, "BaseMetadata") ? "1" : "0";
}
echo "\n";
spl_autoload_register(function($name) {
    echo "autoload:", $name, "\n";
});
echo is_subclass_of("MissingMetadata", "BaseMetadata") ? "1" : "0";
echo is_subclass_of("MissingMetadataNoAutoload", "BaseMetadata", false) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "warning:2:Array to string conversion\n\
interpolated:Array\n\
TypeError: property_exists(): Argument #1 ($object_or_class) must be of type object|string, array given\n\
TypeError: property_exists(): Argument #1 ($object_or_class) must be of type object|string, int given\n\
TypeError: property_exists(): Argument #1 ($object_or_class) must be of type object|string, float given\n\
TypeError: property_exists(): Argument #1 ($object_or_class) must be of type object|string, true given\n\
TypeError: property_exists(): Argument #1 ($object_or_class) must be of type object|string, null given\n\
TypeError: method_exists(): Argument #1 ($object_or_class) must be of type object|string, false given\n\
TypeError: method_exists(): Argument #2 ($method) must be of type string, array given\n\
110011\n0000001\n\
autoload:MissingMetadata\n\
00"
    );
    assert_eq!(execution.stderr, "");
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
fn existence_helpers_accept_fully_qualified_lookup_strings() {
    let execution = run_source(
        r#"<?php
namespace Test\Lookup;

class Box {}
interface Face {}
trait Mix {}
function helper() {}

echo class_exists("Test\\Lookup\\Box") ? "1" : "0";
echo class_exists("\\Test\\Lookup\\Box") ? "1" : "0";
echo interface_exists("Test\\Lookup\\Face") ? "1" : "0";
echo interface_exists("\\Test\\Lookup\\Face") ? "1" : "0";
echo trait_exists("Test\\Lookup\\Mix") ? "1" : "0";
echo trait_exists("\\Test\\Lookup\\Mix") ? "1" : "0";
echo function_exists("Test\\Lookup\\helper") ? "1" : "0";
echo function_exists("\\Test\\Lookup\\helper") ? "1" : "0";
echo function_exists("\\strlen") ? "1" : "0";
echo function_exists("helper") ? "1" : "0";
echo "\n";

spl_autoload_register(function ($class_name) {
    echo "autoload:$class_name\n";
});
var_dump(interface_exists("\\Test\\Lookup\\MissingFace"));
var_dump(trait_exists("\\Test\\Lookup\\MissingMix"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1111111110\nautoload:Test\\Lookup\\MissingFace\nbool(false)\nautoload:Test\\Lookup\\MissingMix\nbool(false)\n"
    );
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
echo extension_loaded("posix") ? "1" : "0";
echo "\n";
$call = "extension_loaded";
echo $call("simplexml") ? "1" : "0";
echo $call("hash") ? "1" : "0";
echo $call("pdo_mysql") ? "1" : "0";
echo $call("posix") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0011111\n0111");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn extension_loaded_coerces_extension_names_and_reports_type_errors() {
    let execution = run_source(
        r#"<?php
set_error_handler(function($_, $message) {
    echo "deprecated:", $message, "\n";
    return true;
});

class ExtensionName {
    public function __toString() {
        return "json";
    }
}

echo extension_loaded(new ExtensionName()) ? "1" : "0";
echo extension_loaded(null) ? "1" : "0";
foreach ([false, true, 42, 3.5] as $name) {
    echo extension_loaded($name) ? "1" : "0";
}
echo "\n";

foreach ([[], new stdClass()] as $name) {
    try {
        extension_loaded($name);
    } catch (Throwable $e) {
        echo $e::class, ": ", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1deprecated:extension_loaded(): Passing null to parameter #1 ($extension) of type string is deprecated\n\
00000\n\
TypeError: extension_loaded(): Argument #1 ($extension) must be of type string, array given\n\
TypeError: extension_loaded(): Argument #1 ($extension) must be of type string, stdClass given\n"
    );
    assert_eq!(execution.stderr, "");
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
        "<?php\necho is_int(1, 2) ? 1 : 0;\n",
        "<?php\necho is_callable(\"strlen\", 1) ? 1 : 0;\n",
        "<?php\necho is_callable(\"strlen\", true, $name) ? 1 : 0;\n",
        "<?php\necho function_exists(42) ? 1 : 0;\n",
        "<?php\necho function_exists(\"strlen\", true) ? 1 : 0;\n",
        "<?php\necho extension_loaded(42) ? 1 : 0;\n",
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
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_folds_direct_function_exists_string_names() {
    let ir = emit_ir_source(
        r#"<?php
$known = "assert";
$missing = "missing_native_function";
$fq_known = "\\assert";
$fq_missing = "\\missing_native_function";

echo function_exists("strlen") ? "1" : "0";
echo function_exists("\\strlen") ? "1" : "0";
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
echo function_exists($fq_known) ? "1" : "0";
echo function_exists($fq_missing) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 12, "{ir}");
    assert_eq!(ir.matches("c\"0\\00\"").count(), 3, "{ir}");
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
echo extension_loaded("posix") ? "1" : "0";
echo extension_loaded($name) ? "1" : "0";
echo "\n";
"#,
    )
    .unwrap();

    assert!(
        ir.contains("@phpc_native_text_membership_with_reference_slot_with_diagnostic"),
        "{ir}"
    );
    for expected in [
        "c\"json\\00\"",
        "c\"hash\\00\"",
        "c\"pdo\\00\"",
        "c\"pdo_mysql\\00\"",
        "c\"posix\\00\"",
    ] {
        assert!(ir.contains(expected), "{ir}");
    }
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
