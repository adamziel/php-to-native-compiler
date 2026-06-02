use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn array_product_accumulates_supported_scalar_values() {
    let source = r#"<?php
$integers = [true, 2, " 4 ", "-3"];
echo array_product($integers), "\n";

$with_zero = [null, true, 2];
echo array_product($with_zero), "\n";

$mixed = [];
$mixed["int"] = 2;
$mixed["float"] = 3.5;
$mixed["exponent"] = "6e1";
$mixed["decimal"] = ".25";
echo array_product($mixed), "\n";

$empty = [];
echo array_product($empty), "\n";
echo $mixed["exponent"], "|", $mixed["decimal"], "\n";

$call = "array_product";
echo $call($mixed);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(execution.stdout, "-24\n0\n105\n1\n6e1|.25\n105");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_product_consumes_64_bit_unsigned_sprintf_values() {
    let execution = run_source(
        r#"<?php
var_dump(array_product([2, sprintf("%u", -1)]));
var_dump(array_product([8.993, 7443241, 988, sprintf("%u", -1) + 0.44]));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "float(3.6893488147419103E+19)\nfloat(1.219953680144986E+30)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_product_non_arrays_raise_catchable_type_errors() {
    let source = r#"<?php
function check($label, $value) {
    try {
        var_dump(array_product($value));
    } catch (TypeError $e) {
        echo $label, ": ", $e->getMessage(), "\n";
    }
}

check("null", null);
check("int", 42);
check("float", 1.25);
check("string", "items");
check("object", new stdClass());
check("true", true);
check("false", false);

$call = "array_product";
try {
    $call(42);
} catch (TypeError $e) {
    echo "dynamic: ", $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "null: array_product(): Argument #1 ($array) must be of type array, null given\n\
int: array_product(): Argument #1 ($array) must be of type array, int given\n\
float: array_product(): Argument #1 ($array) must be of type array, float given\n\
string: array_product(): Argument #1 ($array) must be of type array, string given\n\
object: array_product(): Argument #1 ($array) must be of type array, stdClass given\n\
true: array_product(): Argument #1 ($array) must be of type array, true given\n\
false: array_product(): Argument #1 ($array) must be of type array, false given\n\
dynamic: array_product(): Argument #1 ($array) must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_product_warns_for_non_numeric_strings() {
    let execution =
        run_source("<?php\n$items = [\"ok\", \"abc\"];\necho array_product($items);\n").unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: array_product(): Multiplication is not supported on type string in Command line code on line 3\n\nWarning: array_product(): Multiplication is not supported on type string in Command line code on line 3\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_product_warns_for_non_scalar_values() {
    let execution = run_source("<?php\n$items = [[]];\necho array_product($items);\n").unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: array_product(): Multiplication is not supported on type array in Command line code on line 3\n1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_product_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_product([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
