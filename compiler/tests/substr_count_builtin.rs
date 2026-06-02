use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn substr_count_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo substr_count("db:3306", ":");
echo "|";
echo substr_count("2001:db8::1", ":");
echo "|";
echo substr_count("aaaa", "aa");
echo "|";
echo substr_count("abcabc", "a", 1);
echo "|";
echo substr_count("abcabc", "a", 0, 3);
echo "|";
echo substr_count("abcabc", "a", 0, -1);
echo "|";
echo substr_count("abc", "c", -1);
echo "|";
echo substr_count("abc", "needle", 3);
echo "|";
echo substr_count(12121, 21);
echo "|";
echo substr_count("this is a string", "t", "5", "10");
echo "|";
$bytes = chr(128) . chr(129) . chr(128) . chr(0) . chr(255) . chr(254) . chr(255);
echo substr_count($bytes, chr(128)) . ":" . substr_count($bytes, chr(255)) . ":" . substr_count($bytes, chr(0));
echo "|";
$long = str_repeat("abcacbabca", 100);
echo substr_count($long, "bca", -200, -50);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|3|2|1|1|2|1|0|2|1|2:2:1|30");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_count_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "substr_count";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("a:b:c", ":");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_count_uses_php_string_argument_boundary_for_haystack_and_needle() {
    let execution = run_source(
        r#"<?php
class Haystack {
    public function __toString() {
        return "aaaa";
    }
}
class Needle {
    public function __toString() {
        return "aa";
    }
}

$call = "substr_count";
echo substr_count(new Haystack(), new Needle()), "|";
echo $call(new Haystack(), new Needle(), false, "4"), "\n";

set_error_handler(function($_, $message) {
    echo "deprecated:", $message, "\n";
});
echo substr_count(null, "x"), "\n";
try {
    substr_count("abc", null);
} catch (ValueError $e) {
    echo "null-needle:", $e->getMessage(), "\n";
}

foreach ([[[], "a"], ["abc", new stdClass()]] as $case) {
    try {
        substr_count($case[0], $case[1]);
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2|2\n",
            "deprecated:substr_count(): Passing null to parameter #1 ($haystack) of type string is deprecated\n",
            "0\n",
            "deprecated:substr_count(): Passing null to parameter #2 ($needle) of type string is deprecated\n",
            "null-needle:substr_count(): Argument #2 ($needle) must not be empty\n",
            "substr_count(): Argument #1 ($haystack) must be of type string, array given\n",
            "substr_count(): Argument #2 ($needle) must be of type string, stdClass given\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_count_rejects_forms_outside_current_subset() {
    let empty_needle = run_source(
        r#"<?php
try {
    substr_count('abc', '');
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        empty_needle.stdout,
        "substr_count(): Argument #2 ($needle) must not be empty"
    );

    let array_haystack = run_source(
        r#"<?php
try {
    substr_count(['abc'], 'a');
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        array_haystack.stdout,
        "substr_count(): Argument #1 ($haystack) must be of type string, array given"
    );

    let array_needle = run_source(
        r#"<?php
try {
    substr_count('abc', ['a']);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        array_needle.stdout,
        "substr_count(): Argument #2 ($needle) must be of type string, array given"
    );

    let bad_offset = run_source(
        r#"<?php
try {
    substr_count('abc', 'a', 'bad');
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        bad_offset.stdout,
        "substr_count(): Argument #3 ($offset) must be of type int, string given"
    );

    let bad_length = run_source(
        r#"<?php
try {
    substr_count('abc', 'a', 0, 'bad');
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        bad_length.stdout,
        "substr_count(): Argument #4 ($length) must be of type int, string given"
    );

    let out_of_bounds = run_source(
        r#"<?php
try {
    substr_count('abc', 'a', 1, 5);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        out_of_bounds.stdout,
        "substr_count(): Argument #4 ($length) must be contained in argument #1 ($haystack)"
    );

    let offset_out_of_bounds = run_source(
        r#"<?php
try {
    substr_count('abc', 'a', -20);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        offset_out_of_bounds.stdout,
        "substr_count(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
    );

    let too_few = run_source("<?php\nsubstr_count('abc');\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for substr_count(): expected 2 to 4 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_substr_count_metadata_and_routes_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("substr_count") ? "1" : "0";
echo is_callable("substr_count") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let routed = emit_ir_source("<?php\nsubstr_count('abc', 'b');\n").unwrap();
    assert!(
        routed.contains(
            "declare %phpc.NativeValueHandle @phpc_native_value_string_search_result_with_diagnostic"
        ),
        "{routed}"
    );
    assert!(
        routed.contains(
            "call %phpc.NativeValueHandle @phpc_native_value_string_search_result_with_diagnostic"
        ),
        "{routed}"
    );
    assert!(routed.contains("i8 1, ptr %"), "{routed}");
    assert!(
        routed.contains("call void @phpc_native_value_free"),
        "{routed}"
    );
}
