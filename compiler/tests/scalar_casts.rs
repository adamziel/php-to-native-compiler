use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source, run_source_with_source_file};

const LLVM_CAST_REJECTION: &str = "LLVM cast lowering rejects (string), (int)/(integer), (bool)/(boolean), (float)/(double), (array), (object), and (void) casts plus strval(), boolval(), floatval(), and doubleval() until native PHP scalar conversion, array/object materialization, warning/recovery behavior, object/resource handling, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded cast behavior";

#[test]
fn string_casts_execute_for_current_scalar_and_null_subset() {
    let execution = run_source(
        r#"<?php
echo "[", (string) null, "]\n";
echo "[", (string) false, "]\n";
echo (STRING) true, "|", (string) 42, "|", (string) 3.5, "|", (string) "ok", "\n";
echo (string) fdiv(0, 0), "\n";
echo ((string) true) === "1" ? "string" : "other";
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: unexpected NAN value was coerced to string"));
    assert!(execution.stdout.ends_with("\nNAN\nstring"));
    assert!(execution.stdout.starts_with("[]\n[]\n1|42|3.5|ok\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn int_casts_execute_for_current_scalar_and_null_subset() {
    let execution = run_source(
        r#"<?php
echo (int) null, "|", (int) false, "|", (int) true, "\n";
echo (integer) 42, "|", (int) -3.8, "|", (int) " 15 ", "|", (int) "2.9", "\n";
echo (int) "", "|", (int) "not numeric", "|", (int) "+.", "|", (int) "128m", "|", (int) "1.2e3m";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|0|1\n42|-3|15|2\n0|0|0|128|1200");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn int_casts_execute_current_leading_numeric_string_subset() {
    let execution = run_source(
        r#"<?php
echo (int) "42abc", "|", (int) "2.9m", "|", (int) "-3kb", "|", (int) "+7foo", "\n";
echo (int) ".5m", "|", (int) "-.5m", "|", (int) "1e3m", "|", (int) "1e";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "42|2|-3|7\n0|0|1000|1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn bool_casts_execute_for_current_value_subset() {
    let execution = run_source(
        r#"<?php
echo (bool) null ? "true" : "false", "|";
echo (boolean) false ? "true" : "false", "|";
echo (bool) true ? "true" : "false", "\n";
echo (bool) 0 ? "true" : "false", "|";
echo (bool) 1 ? "true" : "false", "|";
echo (bool) 0.0 ? "true" : "false", "|";
echo (bool) -0.5 ? "true" : "false", "\n";
echo (bool) "" ? "true" : "false", "|";
echo (bool) "0" ? "true" : "false", "|";
echo (bool) "false" ? "true" : "false", "\n";
echo (bool) [] ? "true" : "false", "|";
echo (bool) [0] ? "true" : "false", "\n";
class Flag {}
echo (bool) new Flag() ? "true" : "false";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "false|false|true\nfalse|true|false|true\nfalse|false|true\nfalse|true\ntrue"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn float_casts_execute_for_current_scalar_and_null_subset() {
    let execution = run_source(
        r#"<?php
echo (float) null, "|", (float) false, "|", (float) true, "\n";
echo (double) 42, "|", (float) -3.8, "|", (float) " 15 ", "|", (float) "2.9", "\n";
echo (float) "", "|", (float) "not numeric", "|", (float) "1e3", "\n";
echo is_float((float) "1") ? "float" : "other", "|";
echo ((double) "2.25") === 2.25 ? "double" : "other";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0|0|1\n42|-3.8|15|2.9\n0|0|1000\nfloat|double"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_casts_execute_array_object_and_resource_warning_paths() {
    let execution = run_source(
        r#"<?php
class StringableBox { public function __toString() { return "box"; } }
echo (string) [1], "|", strval(new StringableBox()), "|", strval(STDIN), "\n";
try {
    strval(new stdClass());
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: Array to string conversion"));
    assert!(execution.stdout.contains("Array|box|Resource id #"));
    assert!(execution
        .stdout
        .contains("Object of class stdClass could not be converted to string"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn int_casts_execute_array_object_and_resource_warning_paths() {
    let execution = run_source(
        r#"<?php
class CountableBox {}
echo (int) [], "|", (int) [1], "|", (int) STDERR, "|", (int) new CountableBox(), "\n";
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: Object of class CountableBox could not be converted to int"));
    assert!(execution.stdout.contains("0|1|3|"));
    assert!(execution.stdout.ends_with("1\n"));
    assert_eq!(execution.exit_code, 0);

    let execution = run_source("<?php\necho (int) \"9223372036854775808x\";\n").unwrap();

    assert_eq!(execution.stdout, "9223372036854775807");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn int_casts_warn_and_wrap_nonrepresentable_float_values() {
    let execution = run_source(
        r#"<?php
$values = [10e120, 10e300, fdiv(0, 0), -4000000000000000000000.];
foreach ($values as $value) {
    var_dump((int) $value);
}
var_dump((int) (string) 10e120);
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: The float 1.0E+121 is not representable as an int, cast occurred"));
    assert!(execution
        .stdout
        .contains("Warning: The float 1.0E+301 is not representable as an int, cast occurred"));
    assert!(execution
        .stdout
        .contains("Warning: The float NAN is not representable as an int, cast occurred"));
    assert!(execution
        .stdout
        .contains("Warning: The float -4.0E+21 is not representable as an int, cast occurred"));
    assert!(execution.stdout.contains("int(2943463994972700672)"));
    assert!(execution.stdout.ends_with("int(9223372036854775807)\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_and_string_offsets_warn_and_wrap_nonrepresentable_float_keys() {
    let execution = run_source(
        r#"<?php
set_error_handler(function ($errno, $errstr) {
    echo $errstr, "\n";
});

$array = [0 => "zero"];
unset($array[1.0E+42]);
var_dump(isset($array[1.0E+42]));
var_dump(array_key_exists(1.0E+42, $array));

$array = [10e120 => "large"];
var_dump($array[10e120]);

$string = "abc";
var_dump($string[10e120]);
$string[10e120] = "Z";
var_dump($string);
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("The float 1.0E+42 is not representable as an int, cast occurred\nbool(false)"));
    assert!(execution
        .stdout
        .contains("The float 1.0E+42 is not representable as an int, cast occurred\nbool(false)"));
    assert!(execution.stdout.contains(
        "The float 1.0E+121 is not representable as an int, cast occurred\n\
The float 1.0E+121 is not representable as an int, cast occurred\n\
string(5) \"large\""
    ));
    assert!(execution
        .stdout
        .contains("String offset cast occurred\nstring(1) \"a\""));
    assert!(execution
        .stdout
        .contains("String offset cast occurred\nstring(3) \"Zbc\""));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn float_casts_execute_array_resource_and_leading_numeric_paths() {
    let execution = run_source(
        r#"<?php
class FloatBox {}
echo (float) [], "|", (float) [1], "|", (float) STDERR, "|", (float) "42abc", "|";
echo "10.0 dollar" + 1, "|", "10.0 dollar" + 1.0, "|", (float) new FloatBox(), "\n";
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: A non-numeric value encountered"));
    assert!(execution
        .stdout
        .contains("Warning: Object of class FloatBox could not be converted to float"));
    assert!(execution.stdout.contains("0|1|3|42|"));
    assert!(execution.stdout.contains("11|"));
    assert!(execution.stdout.ends_with("1\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn scalar_conversion_builtins_execute_current_cast_subset() {
    let execution = run_source(
        "<?php\n\
echo strval(\"A\"), \"|\", (boolval(\"0\") ? \"true\" : \"false\"), \"|\";\n\
echo intval(\"0b101\", 0), \"|\", intval(\"0b101\", 2), \"|\", intval(\"0b101\"), \"|\";\n\
echo floatval(\"10.2 dollars\"), \"|\", doubleval(true), \"\\n\";\n\
echo bin2hex(strval(\"\\x80\\xff\")), \"\\n\";\n\
echo function_exists(\"intval\") ? \"exists\" : \"missing\", \"|\";\n\
echo is_callable(\"floatval\") ? \"callable\" : \"not-callable\", \"\\n\";\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "A|false|5|5|0|10.2|1\n80ff\nexists|callable\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn intval_base_argument_coercions_match_current_php_subset() {
    let execution = run_source(
        r#"<?php
echo intval("101", "2"), "|";
echo intval("101", 2.0), "|";
echo intval("101", true), "|";
echo intval("101", 37), "|";
echo intval(101, 37), "\n";
try {
    intval("101", "2abc");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "5|5|0|0|101\nintval(): Argument #2 ($base) must be of type int, string given\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_casts_execute_for_current_null_scalar_and_array_subset() {
    let execution = run_source(
        r#"<?php
print_r((array) null);
print_r((array) false);
print_r((array) 42);
$items = ["name" => "Ada"];
print_r((array) $items);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Array\n(\n)\nArray\n(\n    [0] => \n)\nArray\n(\n    [0] => 42\n)\nArray\n(\n    [name] => Ada\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_casts_materialize_initialized_object_properties_with_mangled_keys() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;
    protected $secret = "S";
    private $token = "T";
}
$box = new Box();
$box->name = "Ada";
$array = (array) $box;
$keys = array_keys($array);
echo count($array), "\n";
echo strlen($keys[0]), "|", $keys[0], "|", $array[$keys[0]], "\n";
echo strlen($keys[1]), "|", $array[$keys[1]], "\n";
echo strlen($keys[2]), "|", $array[$keys[2]], "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3\n4|name|Ada\n9|S\n10|T\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn settype_direct_variables_execute_current_cast_subset() {
    let execution = run_source(
        r#"<?php
$int = "8754456";
var_dump(settype($int, "int"));
var_dump($int);
$float = "10.25";
settype($float, "double");
var_dump($float);
$bool = "0";
settype($bool, "boolean");
var_dump($bool);
$array = "x";
settype($array, "array");
echo count($array), "|", $array[0], "\n";
$object = ["a" => 1];
settype($object, "object");
echo get_class($object), "|", $object->a, "\n";
$null = true;
settype($null, "null");
var_dump($null);
$kept = "kept";
try {
    settype($kept, "resource");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
var_dump($kept);
echo function_exists("settype") ? "exists" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nint(8754456)\nfloat(10.25)\nbool(false)\n1|x\nstdClass|1\nNULL\nCannot convert to resource type\nstring(4) \"kept\"\nexists"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn settype_handles_undefined_variable_validation_and_string_failure_side_effect() {
    let execution = run_source(
        r#"<?php
try {
    settype($missing, "unknown");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
var_dump(isset($missing));
settype($stringDefault, "string");
var_dump($stringDefault);
settype($intDefault, "integer");
var_dump($intDefault);
$object = new stdClass();
try {
    settype($object, "string");
} catch (Error $e) {
    echo "Error: ", $e->getMessage(), "\n";
}
var_dump($object);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "settype(): Argument #2 ($type) must be a valid type\nbool(false)\nstring(0) \"\"\nint(0)\nError: Object of class stdClass could not be converted to string\nstring(0) \"\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn settype_nan_scalar_targets_warn_and_ignore_handler_mutations() {
    let execution = run_source(
        r#"<?php
$mode = "";
set_error_handler(function ($errno, $errstr) {
    global $nan, $mode;
    if ($mode === "null") {
        $nan = null;
    } elseif ($mode === "unset") {
        unset($nan);
    } else {
        $nan = "changed";
    }
    echo $errstr, "\n";
});

$mode = "null";
$nan = fdiv(0, 0);
settype($nan, "bool");
var_dump($nan);

$mode = "unset";
$nan = fdiv(0, 0);
settype($nan, "string");
var_dump($nan);

$mode = "changed";
$nan = fdiv(0, 0);
settype($nan, "null");
var_dump($nan);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "unexpected NAN value was coerced to bool\nbool(true)\n\
unexpected NAN value was coerced to string\nstring(3) \"NAN\"\n\
unexpected NAN value was coerced to null\nNULL\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn settype_nan_array_and_object_targets_wrap_handler_value_or_original() {
    let execution = run_source(
        r#"<?php
$mode = "";
set_error_handler(function ($errno, $errstr) {
    global $nan, $mode;
    if ($mode === "null") {
        $nan = null;
    } elseif ($mode === "unset") {
        unset($nan);
    } else {
        $nan = "changed";
    }
    echo $errstr, "\n";
});

$mode = "null";
$nan = fdiv(0, 0);
settype($nan, "array");
echo gettype($nan[0]), "\n";

$mode = "unset";
$nan = fdiv(0, 0);
settype($nan, "array");
echo is_nan($nan[0]) ? "array-nan\n" : "array-other\n";

$mode = "changed";
$nan = fdiv(0, 0);
settype($nan, "object");
echo get_class($nan), ":", $nan->scalar, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "unexpected NAN value was coerced to array\nNULL\n\
unexpected NAN value was coerced to array\narray-nan\n\
unexpected NAN value was coerced to object\nstdClass:changed\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn remaining_casts_have_stable_parse_error() {
    let error = run_source("<?php\necho (unset) \"1\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported cast expression: only (string), (int), (bool), (float), (array), and (object) casts are implemented"
    );
}

#[test]
fn void_cast_discards_values_and_suppresses_no_discard_warning() {
    let execution = run_source_with_source_file(
        r#"<?php
class WithDestructor {
    public function __destruct() {
        echo "WithDestructor::__destruct\n";
    }
}

function make_with_destructor() {
    return new WithDestructor();
}

$count = 0;

#[NoDiscard]
function incCount() {
    global $count;
    $count++;
    return $count;
}

echo "Before\n";
(void)make_with_destructor();
echo "After\n";

for ($count = 0, (void)incCount(), incCount(); (void)incCount(), incCount() < 6; incCount(), $count++, incCount(), (void)incCount()) {
    echo $count . "\n";
}
"#,
        "/tmp/void_cast.php",
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Before\nWithDestructor::__destruct\nAfter\n"));
    assert_eq!(
        execution
            .stdout
            .matches("The return value of function incCount() should either be used or intentionally ignored by casting it as (void)")
            .count(),
        3
    );
    assert!(execution
        .stdout
        .contains("After\n\nWarning: The return value"));
    assert!(execution.stdout.contains("Warning: The return value of function incCount() should either be used or intentionally ignored by casting it as (void) in /tmp/void_cast.php on line "));
    assert!(execution
        .stdout
        .contains("\n4\n\nWarning: The return value of function incCount() should either be used"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn void_cast_has_statement_only_parse_boundary() {
    let error = run_source("<?php\n$tmp = (void)$dummy;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 2);
    assert_eq!(error.message, "syntax error, unexpected token \"(void)\"");

    let error = run_source("<?php\nfor (;(void)true;);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 2);
    assert_eq!(
        error.message,
        "syntax error, unexpected token \";\", expecting \",\""
    );
}

#[test]
fn emit_ir_rejects_void_cast_statement_until_native_cast_lowering_exists() {
    let error = emit_ir_source("<?php\n(void) 42;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CAST_REJECTION);
}

#[test]
fn emit_ir_lowers_current_static_scalar_cast_subset() {
    let ir = emit_ir_source(
        r#"<?php
$payload = "15";
echo (string) null, (string) false, (string) true, (string) 42, (string) "ok", "\n";
echo (string) 2.5, "\n";
echo (int) null, "|", (int) false, "|", (int) true, "|", (int) $payload, "|", (int) "2.9", "|", (int) "word", "\n";
echo (bool) null ? "T" : "F";
echo (bool) "0" ? "T" : "F";
echo (bool) "value" ? "T" : "F";
echo "\n";
echo (float) null, "|", (float) false, "|", (float) true, "|", (float) 42, "|", (float) "2.5", "|", (float) "word";
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"1\\00\""), "{ir}");
    assert!(ir.contains("c\"42\\00\""), "{ir}");
    assert!(ir.contains("c\"ok\\00\""), "{ir}");
    assert!(ir.contains("c\"2.5\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 15)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 2)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 0)"), "{ir}");
    assert!(ir.contains("sitofp i64 42 to double"), "{ir}");
    assert!(ir.contains("@phpc_native_float(double 2.5)"), "{ir}");
    assert!(ir.contains("@phpc_native_float(double 0.0)"), "{ir}");
    assert!(!ir.contains("LLVM cast lowering rejects"), "{ir}");
}

#[test]
fn emit_ir_rejects_remaining_cast_edges_until_native_cast_lowering_exists() {
    let error = emit_ir_source("<?php\necho (string) [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CAST_REJECTION);

    let error = emit_ir_source("<?php\necho (int) \"42tail\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CAST_REJECTION);

    let error = emit_ir_source("<?php\necho (float) \"42tail\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CAST_REJECTION);

    let error = emit_ir_source("<?php\necho (array) 42;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CAST_REJECTION);

    let error = emit_ir_source("<?php\necho strval(\"value\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CAST_REJECTION);
}

#[test]
fn emit_asm_rejects_array_cast_until_llvm_native_cast_lowering_exists() {
    let error = emit_asm_source("<?php\necho (array) 42;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CAST_REJECTION);
}
