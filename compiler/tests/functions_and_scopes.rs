use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::{run_source, run_source_with_source_file};
use std::path::Path;
use std::process::Command;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

fn system_php_available() -> bool {
    Command::new("php").arg("-v").output().is_ok()
}

fn assert_system_php_fixture_matches_stdout(fixture: &str, expected: &str) {
    if !system_php_available() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("php")
        .arg(manifest_dir.join(fixture))
        .output()
        .expect("run system PHP fixture");
    let expected_stdout =
        std::fs::read_to_string(manifest_dir.join(expected)).expect("read expected stdout");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected_stdout.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

fn assert_run_source_fixture_matches_stdout(source: &str, expected: &str) {
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

fn assert_run_source_fixture_path_matches_stdout(fixture: &str, expected: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(manifest_dir.join(fixture)).expect("read fixture source");
    let expected_stdout =
        std::fs::read_to_string(manifest_dir.join(expected)).expect("read expected stdout");

    assert_run_source_fixture_matches_stdout(&source, &expected_stdout);
}

#[test]
fn user_functions_use_local_scope_without_clobbering_globals() {
    let execution = run_source(
        r#"<?php
$value = "global";
function shadow($value) {
    $value = $value . "-local";
    echo $value, "\n";
}
shadow("arg");
echo $value, "\n";
function make_local() {
    $value = "function";
    return $value;
}
echo make_local(), "\n";
echo $value, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "arg-local\nglobal\nfunction\nglobal\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn user_functions_do_not_import_global_variables_implicitly() {
    let error = runtime_error(
        r#"<?php
$value = "global";
function read_value() {
    return $value;
}
echo read_value();
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 12);
    assert_eq!(error.message, "undefined variable '$value'");
}

#[test]
fn function_scope_global_declaration_imports_existing_global_value() {
    let execution = run_source(
        r#"<?php
$value = 1;
function read_global() {
    global $value;
    return $value;
}
echo read_global();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_scope_global_declaration_writes_back_to_global_scope() {
    let execution = run_source(
        r#"<?php
$count = 1;
function bump() {
    global $count;
    $count = $count + 1;
}
bump();
echo $count;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_scope_global_declaration_materializes_missing_global_as_null() {
    let execution = run_source(
        r#"<?php
function make_missing() {
    global $missing;
    var_dump($missing);
    $missing = 3;
}
make_missing();
var_dump($missing);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "NULL\nint(3)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_scope_global_declaration_shares_values_across_calls() {
    let execution = run_source(
        r#"<?php
$value = "old";
function writer() {
    global $value;
    $value = "new";
}
function reader() {
    global $value;
    return $value;
}
writer();
echo reader();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "new");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn global_import_rebinds_name_after_local_array_offset_alias() {
    let execution = run_source(
        r#"<?php
$value = "root";

function rebind() {
    $local = ["slot" => "local"];
    $value =& $local["slot"];
    $value = "local-mutated";
    echo $value, "|";
    global $value;
    echo $value, "|";
    $value = "updated-root";
}

rebind();
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "local-mutated|root|updated-root");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_imported_global_drops_local_import_without_removing_global_value() {
    let execution = run_source(
        r#"<?php
$value = "global";
function reset_local() {
    global $value;
    unset($value);
    $value = "local";
    echo $value, "\n";
}
reset_local();
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "local\nglobal");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn recursive_user_functions_can_return_values() {
    let execution = run_source(
        r#"<?php
function factorial($n) {
    if ($n <= 1) {
        return 1;
    }
    return $n * factorial($n - 1);
}
echo factorial(5), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "120\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn user_function_default_parameters_are_used_for_omitted_arguments() {
    let execution = run_source(
        r#"<?php
function greet($name = "world", $suffix = "!") {
    echo "hello ", $name, $suffix, "\n";
}
greet();
greet("Ada");
greet("Ada", ".");
function scale($value, $factor = 2, $offset = 1) {
    return $value * $factor + $offset;
}
echo scale(3), "\n";
echo scale(3, 4), "\n";
echo scale(3, 4, 5), "\n";
function default_items($items = ["first", "second" => 2]) {
    echo count($items), ":", $items[0], ":", $items["second"], "\n";
}
default_items();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "hello world!\nhello Ada!\nhello Ada.\n7\n13\n17\n2:first:2\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_declarations_accept_optional_trailing_commas_in_parameter_lists() {
    let execution = run_source(
        r#"<?php
function identity($value,) {
    return $value;
}
function label($name, $suffix = "!",) {
    return $name . $suffix;
}
class Box {
    public function method($value,) {
        return $value;
    }
}
echo identity("Ada"), "\n";
echo label("Grace"), "\n";
echo label("Lin", ".");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada\nGrace!\nLin.");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn empty_parameter_before_trailing_comma_still_fails_parse() {
    let error = parse_error(
        r#"<?php
function invalid(,) {
    return null;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 18);
    assert_eq!(error.message, "expected parameter name");
}

#[test]
fn calls_accept_optional_trailing_commas_in_argument_lists() {
    let execution = run_source(
        r#"<?php
function greet($name, $suffix = "!") {
    return "hello " . $name . $suffix;
}
echo greet("Ada",), "\n";
echo greet("Lin", ".",), "\n";
echo strlen("native",), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello Ada!\nhello Lin.\n6\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn empty_call_argument_before_trailing_comma_still_fails_parse() {
    let error = parse_error(
        r#"<?php
function greet($name) {
    return $name;
}
echo greet(,);
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 12);
    assert_eq!(error.message, "expected expression, found ,");
}

#[test]
fn default_parameter_values_can_reference_global_constants() {
    let execution = run_source(
        r#"<?php
define("RUNTIME_FACTOR", 3);
const BASE = "compiler";
function describe($label = BASE . ":" . ARRAY_FILTER_USE_KEY, $factor = RUNTIME_FACTOR + 1, $items = [BASE => ARRAY_FILTER_USE_BOTH]) {
    echo $label, "|", $factor, "|", $items["compiler"], "\n";
}
describe();
function late_default($value = LATE_DEFAULT) {
    return $value;
}
const LATE_DEFAULT = "late";
echo late_default(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "compiler:2|4|1\nlate\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn undefined_default_parameter_constant_reports_runtime_error_when_omitted() {
    let error = runtime_error(
        r#"<?php
function missing_default($value = MISSING_DEFAULT) {
    return $value;
}
echo missing_default();
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 35);
    assert_eq!(error.message, "undefined constant MISSING_DEFAULT");
}

#[test]
fn default_parameter_arity_errors_report_supported_range() {
    let too_few = runtime_error(
        r#"<?php
function label($value, $suffix = "!") {
    return $value . $suffix;
}
echo label();
"#,
    );

    assert_eq!(too_few.line, 5);
    assert_eq!(too_few.column, 6);
    assert_eq!(
        too_few.message,
        "arity mismatch for label(): expected 1 to 2 argument(s), got 0"
    );

    let too_many = runtime_error(
        r#"<?php
function label($value, $suffix = "!") {
    return $value . $suffix;
}
echo label("a", "b", "c");
"#,
    );

    assert_eq!(too_many.line, 5);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for label(): expected 1 to 2 argument(s), got 3"
    );
}

#[test]
fn default_parameter_values_must_be_constant_expressions_in_current_subset() {
    let error = parse_error(
        r#"<?php
$fallback = "value";
function invalid($value = $fallback) {
    return $value;
}
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 27);
    assert_eq!(
        error.message,
        "default parameter values only support constant expressions in the current subset"
    );
}

#[test]
fn required_parameters_after_defaults_are_rejected_in_current_subset() {
    let error = parse_error(
        r#"<?php
function invalid($first = 1, $second) {
    return $second;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 30);
    assert_eq!(
        error.message,
        "required parameter cannot follow a default parameter in the current subset"
    );
}

#[test]
fn variadic_parameters_collect_extra_arguments() {
    let execution = run_source(
        r#"<?php
function collect($first, ...$items) {
    echo $first, ":", count($items), ":", $items[0], ":", $items[1], "\n";
}
collect("a", "b", "c");
function empty_rest(...$items) {
    echo count($items), "\n";
}
empty_rest();
function optional_rest($first = "x", ...$items) {
    echo $first, ":", count($items), "\n";
}
optional_rest();
optional_rest("y", "z");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "a:2:b:c\n0\nx:0\ny:1\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn variadic_parameters_still_require_non_variadic_arguments() {
    let error = runtime_error(
        r#"<?php
function need_first($first, ...$items) {
    return count($items);
}
need_first();
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "arity mismatch for need_first(): expected at least 1 argument(s), got 0"
    );
}

#[test]
fn typed_and_reference_function_declarations_register_without_invocation() {
    let execution = run_source(
        r#"<?php
function _wp_scan_utf8(string $bytes, int &$at, int &$invalid_length, ?int $max_bytes = null, ?int $max_code_points = null, ?bool &$has_noncharacters = null): int {
    return 0;
}
function union_result($value): int|string {
    return $value;
}
function intersection_param(Iterator&Countable $value) {
    return $value;
}
echo function_exists("_wp_scan_utf8"), "\n";
echo function_exists("union_result"), "\n";
echo function_exists("intersection_param"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\n1\n1\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn typed_function_invocation_is_rejected_until_type_enforcement_exists() {
    let error = runtime_error(
        r#"<?php
function label(string $value): string {
    return $value;
}
echo label("Ada");
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call label(): parameter and return type enforcement is not implemented"
    );
}

#[test]
fn direct_variable_reference_parameter_invocation_copies_back_current_value() {
    let execution = run_source(
        r#"<?php
function mutate(&$value) {
    $value = 2;
    return "done";
}
$value = 1;
echo mutate($value), "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "done|2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_variable_reference_parameter_shares_caller_cell_during_execution() {
    let execution = run_source(
        r#"<?php
function observe() {
    global $value;
    echo "seen=", $value, "|";
}
function mutate(&$param) {
    $param = 2;
    observe();
    $param = 3;
}
$value = 1;
mutate($value);
echo "final=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seen=2|final=3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_reference_parameter_detaches_local_name_from_caller_cell() {
    let execution = run_source(
        r#"<?php
function detach(&$param) {
    unset($param);
    $param = 9;
    echo "local=", $param, "|";
}
$value = 1;
detach($value);
echo "caller=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "local=9|caller=1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_parameter_invocation_rejects_non_variable_arguments() {
    let error = runtime_error(
        r#"<?php
function mutate(&$value) {
    $value = 2;
}
mutate(1);
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 8);
    assert_eq!(
        error.message,
        "unsupported call mutate(): reference parameter invocation is only implemented for direct variable, direct array-offset, direct public object-property array-offset, and bounded magic __get reference arguments in the current subset"
    );
}

#[test]
fn reference_parameters_share_direct_alias_and_array_slot_container_identity() {
    let execution = run_source(
        r#"<?php
function pair_refcow_1657(&$left, &$right) {
    $left = $left . ":left";
    echo $left, "|", $right, "\n";
    $right = $right . ":right";
    echo $left, "|", $right, "\n";
}

$items = array("slot" => "seed");
$alias =& $items["slot"];
pair_refcow_1657($alias, $items["slot"]);
echo $alias, "|", $items["slot"], "\n";

class RefCow1657Bag {
    public $items = array("slot" => "box");
}

$bag = new RefCow1657Bag();
$propertyAlias =& $bag->items["slot"];
pair_refcow_1657($propertyAlias, $bag->items["slot"]);
echo $propertyAlias, "|", $bag->items["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:left|seed:left\nseed:left:right|seed:left:right\nseed:left:right|seed:left:right\nbox:left|box:left\nbox:left:right|box:left:right\nbox:left:right|box:left:right"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn omitted_optional_reference_parameters_use_default_without_reference_binding() {
    let execution = run_source(
        r#"<?php
function cache_get($key, &$found = null) {
    echo isset($found) ? "found-set" : "found-null";
    return $key;
}

echo "|", cache_get("notoptions");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "|found-nullnotoptions");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn method_reference_parameter_invocation_copies_back_direct_variable() {
    let execution = run_source(
        r#"<?php
class Cache {
    public function get($key, &$found = null) {
        $found = false;
        return $key;
    }
}

$cache = new Cache();
$found = null;
echo $cache->get("notoptions", $found), "|", $found === false ? "miss" : "hit";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "notoptions|miss");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn static_local_declarations_persist_values_across_user_function_calls() {
    let execution = run_source(
        r#"<?php
function counter() {
    static $count = 0;
    $count = $count + 1;
    return $count;
}

function once() {
    static $flag;
    if (isset($flag)) {
        return "set";
    }
    $flag = true;
    return "first";
}

echo counter(), "|", counter(), "\n";
echo once(), "|", once();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|2\nfirst|set");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_line_constant_evaluates_from_expression_source_span() {
    let execution = run_source(
        r#"<?php
echo __LINE__, "\n";
$line = __LINE__;
echo $line, "\n";
function default_line($line = __LINE__) {
    echo $line, "\n";
    echo __LINE__, "\n";
}
const DECLARED_LINE = __LINE__;
default_line();
echo DECLARED_LINE, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "2\n3\n5\n7\n9\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_magic_line_until_native_source_mapping_exists() {
    let error = emit_ir_source("<?php\necho __LINE__;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert!(error.message.contains("LLVM magic-constant lowering"));
    assert!(error.message.contains("__LINE__"));
}

#[test]
fn magic_file_constant_evaluates_from_current_source_file_when_available() {
    let execution = run_source_with_source_file(
        r#"<?php
echo __FILE__, "\n";
$file = __FILE__;
echo $file, "\n";
function default_file($file = __FILE__) {
    echo $file, "\n";
}
const DECLARED_FILE = __FILE__;
default_file();
echo DECLARED_FILE, "\n";
"#,
        "virtual/input.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "virtual/input.php\nvirtual/input.php\nvirtual/input.php\nvirtual/input.php\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_magic_file_until_native_source_mapping_exists() {
    let error = emit_ir_source("<?php\necho __FILE__;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert!(error.message.contains("LLVM magic-constant lowering"));
    assert!(error.message.contains("__FILE__"));
}

#[test]
fn magic_dir_constant_evaluates_from_current_source_file_directory_when_available() {
    let execution = run_source_with_source_file(
        r#"<?php
echo __DIR__, "\n";
$dir = __DIR__;
echo $dir, "\n";
function default_dir($dir = __DIR__) {
    echo $dir, "\n";
}
const DECLARED_DIR = __DIR__;
default_dir();
echo DECLARED_DIR, "\n";
"#,
        "virtual/input.php",
    )
    .unwrap();

    assert_eq!(execution.stdout, "virtual\nvirtual\nvirtual\nvirtual\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_dir_constant_uses_dot_for_source_file_without_parent_directory() {
    let execution =
        run_source_with_source_file("<?php\necho __DIR__, \"\\n\";\n", "input.php").unwrap();

    assert_eq!(execution.stdout, ".\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_magic_dir_until_native_source_mapping_exists() {
    let error = emit_ir_source("<?php\necho __DIR__;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert!(error.message.contains("LLVM magic-constant lowering"));
    assert!(error.message.contains("__DIR__"));
}

#[test]
fn magic_function_constant_evaluates_from_current_user_function_context() {
    let execution = run_source(
        r#"<?php
echo "top:", __FUNCTION__, "\n";
function current_name($default = __FUNCTION__) {
    echo "default:", $default, "\n";
    echo "body:", __FUNCTION__, "\n";
}
function caller() {
    current_name();
    echo "caller:", __FUNCTION__, "\n";
}
current_name("manual");
caller();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "top:\ndefault:manual\nbody:current_name\ndefault:current_name\nbody:current_name\ncaller:caller\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_magic_function_until_native_source_mapping_exists() {
    let error = emit_ir_source("<?php\necho __FUNCTION__;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert!(error.message.contains("LLVM magic-constant lowering"));
    assert!(error.message.contains("__FUNCTION__"));
}

#[test]
fn magic_namespace_constant_is_rejected_until_namespace_resolution_exists() {
    let error = parse_error(
        r#"<?php
echo __NAMESPACE__;
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported magic constant __NAMESPACE__: namespace context evaluation requires namespace-aware name resolution, which is not implemented"
    );
}

#[test]
fn magic_trait_constant_is_rejected_until_trait_context_tracking_exists() {
    let error = parse_error(
        r#"<?php
class Box {
    public function label() {
        return __TRAIT__;
    }
}
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 16);
    assert_eq!(
        error.message,
        "unsupported magic constant __TRAIT__: trait context evaluation requires original trait method context tracking through class composition, which is not implemented"
    );
}

#[test]
fn magic_class_constant_evaluates_from_current_class_context() {
    let execution = run_source(
        r#"<?php
echo "top:", __CLASS__, "\n";
function label() {
    return __CLASS__;
}
echo "function:", label(), "\n";
class Box {
    public function label() {
        return __CLASS__;
    }
}
$box = new Box();
echo "method:", $box->label(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "top:\nfunction:\nmethod:Box\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_method_constant_evaluates_from_current_function_or_method_context() {
    let execution = run_source(
        r#"<?php
echo "top:", __METHOD__, "\n";
function free_function($default = __METHOD__) {
    echo "function-default:", $default, "\n";
    echo "function-body:", __METHOD__, "\n";
}
class ParentBox {
    public function inherited() {
        return __METHOD__;
    }
}
class Box extends ParentBox {
    public function label($default = __METHOD__) {
        echo "method-default:", $default, "\n";
        echo "method-body:", __METHOD__, "\n";
    }
    public static function staticLabel() {
        echo "static-body:", __METHOD__, "\n";
    }
}
free_function();
$box = new Box();
$box->label();
echo "inherited:", $box->inherited(), "\n";
Box::staticLabel();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "top:\nfunction-default:free_function\nfunction-body:free_function\nmethod-default:Box::label\nmethod-body:Box::label\ninherited:ParentBox::inherited\nstatic-body:Box::staticLabel\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_return_declarations_inside_unexecuted_body_are_registered() {
    let execution = run_source(
        r#"<?php
function &identity($value) {
    return $value;
}
echo "registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "registered");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_variable_reference_return_assignment_binds_returned_cell() {
    let execution = run_source(
        r#"<?php
function &identity(&$value) {
    return $value;
}

$value = 1;
$alias =& identity($value);
$alias = 2;
echo "value=", $value, "|";
$value = 3;
echo "alias=", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "value=2|alias=3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unset_reference_return_alias_detaches_only_local_name() {
    let execution = run_source(
        r#"<?php
function &identity(&$value) {
    return $value;
}

$value = 1;
$alias =& identity($value);
unset($alias);
$alias = 9;
echo "alias=", $alias, "|value=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "alias=9|value=1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_function_reference_return_assignment_binds_array_offset_arguments() {
    let execution = run_source(
        r#"<?php
function &tag_ref(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$_REQUEST["payload"] = ["slot" => "request"];
$request_alias =& tag_ref($_REQUEST["payload"]["slot"], "function");
$request_alias = $request_alias . ":alias";
echo $_REQUEST["payload"]["slot"], "|", $request_alias, "\n";

$items = ["outer" => ["slot" => "array"]];
$array_alias =& tag_ref($items["outer"]["slot"], "function");
$array_alias = $array_alias . ":alias";
echo $items["outer"]["slot"], "|", $array_alias, "\n";

class WP_Object_Cache {
    public $cache = [];
}
$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$cache_alias =& tag_ref($cache->cache["options"]["alloptions"], "function");
$cache_alias = $cache_alias . ":alias";
echo $cache->cache["options"]["alloptions"], "|", $cache_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request:function:alias|request:function:alias\narray:function:alias|array:function:alias\ncold:function:alias|cold:function:alias"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_return_assignment_binds_returned_array_offset_from_covered_arguments() {
    let execution = run_source(
        r#"<?php
function &pick_slot(&$items, $key, $suffix) {
    $items[$key] = $items[$key] . ":" . $suffix;
    return $items[$key];
}

class SlotPicker {
    public $cache = [];

    public function &pick(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }

    public static function &pickStatic(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }
}

$_REQUEST["payload"] = ["slot" => "request"];
$request_alias =& pick_slot($_REQUEST["payload"], "slot", "function");
$request_alias = $request_alias . ":alias";
echo $_REQUEST["payload"]["slot"], "|", $request_alias, "\n";

$items = ["outer" => ["slot" => "array"]];
$array_alias =& SlotPicker::pickStatic($items["outer"], "slot", "static");
$array_alias = $array_alias . ":alias";
echo $items["outer"]["slot"], "|", $array_alias, "\n";

$picker = new SlotPicker();
$picker->cache["options"]["alloptions"] = "cold";
$cache_alias =& $picker->pick($picker->cache["options"], "alloptions", "method");
$cache_alias = $cache_alias . ":alias";
echo $picker->cache["options"]["alloptions"], "|", $cache_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request:function:alias|request:function:alias\narray:static:alias|array:static:alias\ncold:method:alias|cold:method:alias"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_return_assignment_binds_returned_array_offset_from_direct_variable_parent() {
    let execution = run_source(
        r#"<?php
function &pick_slot_from_parent(&$items, $key, $suffix) {
    $items[$key] = $items[$key] . ":" . $suffix;
    return $items[$key];
}

class DirectParentSlotPicker {
    public function &pick(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }

    public static function &pickStatic(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }
}

$function_items = ["slot" => "function"];
$function_alias =& pick_slot_from_parent($function_items, "slot", "direct");
$function_alias = $function_alias . ":alias";
echo $function_items["slot"], "|", $function_alias, "\n";

$static_items = ["slot" => "static"];
$static_alias =& DirectParentSlotPicker::pickStatic($static_items, "slot", "direct");
$static_alias = $static_alias . ":alias";
echo $static_items["slot"], "|", $static_alias, "\n";

$method_items = ["slot" => "method"];
$picker = new DirectParentSlotPicker();
$method_alias =& $picker->pick($method_items, "slot", "direct");
$method_alias = $method_alias . ":alias";
echo $method_items["slot"], "|", $method_alias, "\n";

$shared_items = ["slot" => "shared"];
$shared_parent =& $shared_items;
$shared_alias =& pick_slot_from_parent($shared_items, "slot", "direct");
$shared_parent["slot"] = $shared_parent["slot"] . ":parent";
echo $shared_items["slot"], "|", $shared_parent["slot"], "|", $shared_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "function:direct:alias|function:direct:alias\nstatic:direct:alias|static:direct:alias\nmethod:direct:alias|method:direct:alias\nshared:direct:parent|shared:direct:parent|shared:direct:parent"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_return_assignment_binds_nested_returned_array_offset_from_direct_variable_parent() {
    let execution = run_source(
        r#"<?php
function &pick_nested_slot_from_parent(&$items, $key, $subkey, $suffix) {
    $items[$key][$subkey] = $items[$key][$subkey] . ":" . $suffix;
    return $items[$key][$subkey];
}

class DirectParentNestedSlotPicker {
    public function &pick(&$items, $key, $subkey, $suffix) {
        $items[$key][$subkey] = $items[$key][$subkey] . ":" . $suffix;
        return $items[$key][$subkey];
    }

    public static function &pickStatic(&$items, $key, $subkey, $suffix) {
        $items[$key][$subkey] = $items[$key][$subkey] . ":" . $suffix;
        return $items[$key][$subkey];
    }
}

$function_items = ["outer" => ["slot" => "function"]];
$function_alias =& pick_nested_slot_from_parent($function_items, "outer", "slot", "direct");
$function_alias = $function_alias . ":alias";
echo $function_items["outer"]["slot"], "|", $function_alias, "\n";

$static_items = ["outer" => ["slot" => "static"]];
$static_alias =& DirectParentNestedSlotPicker::pickStatic($static_items, "outer", "slot", "direct");
$static_alias = $static_alias . ":alias";
echo $static_items["outer"]["slot"], "|", $static_alias, "\n";

$method_items = ["outer" => ["slot" => "method"]];
$picker = new DirectParentNestedSlotPicker();
$method_alias =& $picker->pick($method_items, "outer", "slot", "direct");
$method_alias = $method_alias . ":alias";
echo $method_items["outer"]["slot"], "|", $method_alias, "\n";

$shared_items = ["outer" => ["slot" => "shared"]];
$shared_parent =& $shared_items;
$shared_alias =& pick_nested_slot_from_parent($shared_items, "outer", "slot", "direct");
$shared_parent["outer"]["slot"] = $shared_parent["outer"]["slot"] . ":parent";
echo $shared_items["outer"]["slot"], "|", $shared_parent["outer"]["slot"], "|", $shared_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "function:direct:alias|function:direct:alias\nstatic:direct:alias|static:direct:alias\nmethod:direct:alias|method:direct:alias\nshared:direct:parent|shared:direct:parent|shared:direct:parent"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_return_assignment_binds_child_slot_from_alias_backed_direct_argument_parent() {
    let execution = run_source(
        r#"<?php
function &pick_payload_slot(&$payload, $key, $suffix) {
    $payload[$key] = $payload[$key] . ":" . $suffix;
    return $payload[$key];
}

class AliasBackedParentPicker {
    public function &pick(&$payload, $key, $suffix) {
        $payload[$key] = $payload[$key] . ":" . $suffix;
        return $payload[$key];
    }

    public static function &pickStatic(&$payload, $key, $suffix) {
        $payload[$key] = $payload[$key] . ":" . $suffix;
        return $payload[$key];
    }
}

$_REQUEST["payload"] = ["slot" => "request"];
$request_payload =& $_REQUEST["payload"];
$request_alias =& pick_payload_slot($request_payload, "slot", "function");
$request_alias = $request_alias . ":alias";
echo $_REQUEST["payload"]["slot"], "|", $request_payload["slot"], "|", $request_alias, "\n";

$items = ["outer" => ["slot" => "array"]];
$outer =& $items["outer"];
$static_alias =& AliasBackedParentPicker::pickStatic($outer, "slot", "static");
$static_alias = $static_alias . ":alias";
echo $items["outer"]["slot"], "|", $outer["slot"], "|", $static_alias, "\n";

$method_items = ["outer" => ["slot" => "method"]];
$method_parent =& $method_items["outer"];
$picker = new AliasBackedParentPicker();
$method_alias =& $picker->pick($method_parent, "slot", "method");
$method_items["outer"]["slot"] = $method_items["outer"]["slot"] . ":root";
echo $method_items["outer"]["slot"], "|", $method_parent["slot"], "|", $method_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request:function:alias|request:function:alias|request:function:alias\narray:static:alias|array:static:alias|array:static:alias\nmethod:method:root|method:method:root|method:method:root"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn method_reference_return_assignment_binds_array_offset_arguments() {
    let execution = run_source(
        r#"<?php
class RefTagger {
    public $cache = [];

    public function &tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }

    public static function &tagStatic(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }

    public function runSelf(&$value) {
        $alias =& self::tagStatic($value, "self");
        $alias = $alias . ":alias";
    }
}

$tagger = new RefTagger();

$_REQUEST["payload"] = ["slot" => "request"];
$request_alias =& $tagger->tag($_REQUEST["payload"]["slot"], "method");
$request_alias = $request_alias . ":alias";
echo $_REQUEST["payload"]["slot"], "|", $request_alias, "\n";

$items = ["outer" => ["slot" => "array"]];
$array_alias =& RefTagger::tagStatic($items["outer"]["slot"], "static");
$array_alias = $array_alias . ":alias";
echo $items["outer"]["slot"], "|", $array_alias, "\n";

$tagger->cache["options"]["alloptions"] = "cold";
$cache_alias =& $tagger->tag($tagger->cache["options"]["alloptions"], "method");
$cache_alias = $cache_alias . ":alias";
echo $tagger->cache["options"]["alloptions"], "|", $cache_alias, "\n";

$self_items = ["slot" => "self"];
$tagger->runSelf($self_items["slot"]);
echo $self_items["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request:method:alias|request:method:alias\narray:static:alias|array:static:alias\ncold:method:alias|cold:method:alias\nself:self:alias"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn method_reference_return_assignment_binds_returned_cell() {
    let execution = run_source(
        r#"<?php
class Box {
    public function &identity(&$value) {
        return $value;
    }
}

$box = new Box();
$value = 1;
$alias =& $box->identity($value);
$alias = 2;
echo "value=", $value, "|";
$value = 3;
echo "alias=", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "value=2|alias=3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn named_static_method_reference_return_assignment_binds_returned_cell() {
    let execution = run_source(
        r#"<?php
class Box {
    public static function &identity(&$value) {
        return $value;
    }
}

$value = 1;
$alias =& Box::identity($value);
$alias = 2;
echo "value=", $value, "|";
$value = 3;
echo "alias=", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "value=2|alias=3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn self_static_method_reference_return_assignment_binds_returned_cell() {
    let execution = run_source(
        r#"<?php
class Box {
    public static function &identity(&$value) {
        return $value;
    }

    public function run(&$value) {
        $alias =& self::identity($value);
        $alias = 2;
    }
}

$box = new Box();
$value = 1;
$box->run($value);
echo "value=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "value=2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parent_static_method_reference_return_assignment_binds_returned_cell() {
    let execution = run_source(
        r#"<?php
class BaseBox {
    public static function &identity(&$value) {
        return $value;
    }
}

class Box extends BaseBox {
    public function run(&$value) {
        $alias =& parent::identity($value);
        $alias = 2;
    }
}

$box = new Box();
$value = 1;
$box->run($value);
echo "value=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "value=2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn late_static_method_reference_return_assignment_binds_returned_cell() {
    let execution = run_source(
        r#"<?php
class BaseBox {
    public static function &identity(&$value) {
        echo "base|";
        return $value;
    }

    public function run(&$value) {
        $alias =& static::identity($value);
        $alias = 2;
    }
}

class Box extends BaseBox {
    public static function &identity(&$value) {
        echo "child|";
        return $value;
    }
}

$value = 1;
$base = new BaseBox();
$base->run($value);
echo "baseValue=", $value, "|";

$value = 3;
$box = new Box();
$box->run($value);
echo "childValue=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "base|baseValue=2|child|childValue=2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_static_receiver_reference_return_assignment_binds_returned_cell() {
    let execution = run_source(
        r#"<?php
class Box {
    public static function &identity(&$value) {
        return $value;
    }
}

$class = "Box";
$value = 1;
$alias =& $class::identity($value);
$alias = 2;
echo "class=", $value, "|";

$box = new Box();
$value = 3;
$alias =& $box::identity($value);
$alias = 4;
echo "object=", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "class=2|object=4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_static_reference_return_assignment_reports_stable_boundary() {
    let error = runtime_error(
        r#"<?php
class Box {
    public static function &__callStatic($method, $args) {
        return $args[0];
    }
}

$value = 1;
$alias =& Box::missing($value);
"#,
    );

    assert_eq!(
        error.message,
        "unsupported call Box::__callStatic(): magic __callStatic reference-return method sources are not implemented"
    );
}

#[test]
fn normal_reference_return_invocation_reads_returned_cell_by_value() {
    let execution = run_source(
        r#"<?php
function &identity($value) {
    return $value;
}
echo identity(1);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn normal_reference_return_invocation_reads_returned_array_offset_by_value() {
    let execution = run_source(
        r#"<?php
function &pick_normal_refcow_slot(&$items, $key, $suffix) {
    $items[$key] = $items[$key] . ":" . $suffix;
    return $items[$key];
}

class NormalRefCowSlotPicker {
    public $cache = [];

    public function &pick(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }

    public static function &pickStatic(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }
}

$_REQUEST["payload"] = ["slot" => "request"];
$payload =& $_REQUEST["payload"];
echo pick_normal_refcow_slot($payload, "slot", "function"), "|", $_REQUEST["payload"]["slot"], "\n";

$items = ["outer" => ["slot" => "array"]];
echo NormalRefCowSlotPicker::pickStatic($items["outer"], "slot", "static"), "|", $items["outer"]["slot"], "\n";

$picker = new NormalRefCowSlotPicker();
$picker->cache["options"]["alloptions"] = "cold";
echo $picker->pick($picker->cache["options"], "alloptions", "method"), "|", $picker->cache["options"]["alloptions"], "\n";

$dynamic = ["slot" => "dynamic"];
$class = "NormalRefCowSlotPicker";
echo $class::pickStatic($dynamic, "slot", "dynamic"), "|", $dynamic["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request:function|request:function\narray:static|array:static\ncold:method|cold:method\ndynamic:dynamic|dynamic:dynamic"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_return_methods_inside_unexecuted_class_are_registered() {
    let execution = run_source(
        r#"<?php
class Factory {
    public function &make() {
        $value = 1;
        return $value;
    }
}
echo "loaded";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "loaded");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn normal_reference_return_method_invocation_reads_returned_cell_by_value() {
    let execution = run_source(
        r#"<?php
class Factory {
    public function &make() {
        $value = 1;
        return $value;
    }
}
$factory = new Factory();
echo $factory->make();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_direct_variable_sources_create_current_alias_cells() {
    let execution = run_source(
        r#"<?php
$value = 1;
$alias =& $value;
$alias = 2;
echo $value;
echo "|";
$value = 3;
echo $alias;
unset($alias);
$value = 4;
echo "|";
echo $value;
$left = 5;
$right =& $left;
unset($left);
echo "|";
echo $right;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "2|3|4|5");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_handle_variable_source_executes_current_subset() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;
}
$box = new Box();
$box->name = "before";
$alias =& $box;
$alias->name = "after";
echo $box->name;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_handle_array_target_executes_current_subset() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;
}
function remember($key) {
    global $items;
    static $box = null;
    if ($box === null) {
        $box = new Box();
    }
    $box->name = "stored";
    $items[$key] =& $box;
    return $items[$key];
}
$value = remember("primary");
echo $value->name;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "stored");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_offset_target_aliases_direct_variable_source() {
    let execution = run_source(
        r#"<?php
$items = ["name" => "Ada"];
$key = "name";
$value = "Grace";
$items[$key] =& $value;
echo $items["name"];
echo "|";
$value = "Hedy";
echo $items["name"];
echo "|";
$items[$key] = "Katherine";
echo $value;
unset($value);
$value = "detached";
echo "|";
echo $items["name"], "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace|Hedy|Katherine|Katherine|detached");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_offset_target_materializes_missing_and_null_roots() {
    let execution = run_source(
        r#"<?php
$items = [];
$source = "created";
$items["missing"] =& $source;
$source = "updated";
echo $items["missing"];
echo "|";
$items["missing"] = "slot";
echo $source;
echo "|";
$root_value = "root";
$undefined["slot"] =& $root_value;
$root_value = "changed";
echo $undefined["slot"];
echo "|";
$nullable = null;
$null_value = "from-null";
$nullable["slot"] =& $null_value;
$nullable["slot"] = "from-slot";
echo $null_value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "updated|slot|changed|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_offset_target_rebinds_direct_alias_group() {
    let execution = run_source(
        r#"<?php
$value = "source";
$other =& $value;
$items["slot"] =& $value;
$value = "from-value";
echo $items["slot"], "|", $other, "|";
$other = "from-other";
echo $items["slot"], "|", $value, "|";
$items["slot"] = "from-slot";
echo $value, "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-value|from-value|from-other|from-other|from-slot|from-slot"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_append_target_aliases_direct_variable_source() {
    let execution = run_source(
        r#"<?php
$items = [2 => "Ada"];
$value = "Grace";
$items[] =& $value;
echo $items[3];
echo "|";
$value = "Hedy";
echo $items[3];
echo "|";
$items[3] = "Katherine";
echo $value;
unset($value);
$value = "detached";
echo "|";
echo $items[3], "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace|Hedy|Katherine|Katherine|detached");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_append_target_materializes_roots_and_source() {
    let execution = run_source(
        r#"<?php
$items = ["name" => "Ada"];
$source = "string-key";
$items[] =& $source;
$source = "zero";
echo $items[0];
echo "|";
$root_value = "root";
$undefined_root[] =& $root_value;
$root_value = "changed";
echo $undefined_root[0];
echo "|";
$nullable = null;
$null_value = "from-null";
$nullable[] =& $null_value;
$nullable[0] = "from-slot";
echo $null_value;
echo "|";
unset($undefined_source);
$targets = [];
$targets[] =& $undefined_source;
$undefined_source = "later";
echo $targets[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "zero|changed|from-slot|later");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_append_target_rebinds_direct_alias_group() {
    let execution = run_source(
        r#"<?php
$value = "source";
$other =& $value;
$items[] =& $value;
$value = "from-value";
echo $items[0], "|", $other, "|";
$other = "from-other";
echo $items[0], "|", $value, "|";
$items[0] = "from-slot";
echo $value, "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-value|from-value|from-other|from-other|from-slot|from-slot"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_offset_target_aliases_direct_variable_source() {
    let execution = run_source(
        r#"<?php
$items = [];
$value = "Grace";
$items["outer"]["name"] =& $value;
echo $items["outer"]["name"];
echo "|";
$value = "Hedy";
echo $items["outer"]["name"];
echo "|";
$items["outer"]["name"] = "Katherine";
echo $value;
unset($value);
$value = "detached";
echo "|";
echo $items["outer"]["name"], "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace|Hedy|Katherine|Katherine|detached");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_offset_target_materializes_roots_and_source() {
    let execution = run_source(
        r#"<?php
$root_value = "root";
$undefined_root["outer"]["slot"] =& $root_value;
$root_value = "changed";
echo $undefined_root["outer"]["slot"];
echo "|";
$nullable = null;
$null_value = "from-null";
$nullable["outer"]["slot"] =& $null_value;
$nullable["outer"]["slot"] = "from-slot";
echo $null_value;
echo "|";
unset($undefined_source);
$targets = [];
$targets["outer"]["slot"] =& $undefined_source;
$undefined_source = "later";
echo $targets["outer"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|from-slot|later");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_offset_target_rebinds_direct_alias_group() {
    let execution = run_source(
        r#"<?php
$value = "source";
$other =& $value;
$items["outer"]["slot"] =& $value;
$value = "from-value";
echo $items["outer"]["slot"], "|", $other, "|";
$other = "from-other";
echo $items["outer"]["slot"], "|", $value, "|";
$items["outer"]["slot"] = "from-slot";
echo $value, "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-value|from-value|from-other|from-other|from-slot|from-slot"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_append_target_aliases_direct_variable_source() {
    let execution = run_source(
        r#"<?php
$items = [];
$value = "Grace";
$items["outer"][] =& $value;
echo $items["outer"][0];
echo "|";
$value = "Hedy";
echo $items["outer"][0];
echo "|";
$items["outer"][0] = "Katherine";
echo $value;
unset($value);
$value = "detached";
echo "|";
echo $items["outer"][0], "|", $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace|Hedy|Katherine|Katherine|detached");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_append_target_materializes_roots_and_source() {
    let execution = run_source(
        r#"<?php
$root_value = "root";
$undefined_root["outer"][] =& $root_value;
$root_value = "changed";
echo $undefined_root["outer"][0];
echo "|";
$nullable = null;
$null_value = "from-null";
$nullable["outer"][] =& $null_value;
$nullable["outer"][0] = "from-slot";
echo $null_value;
echo "|";
unset($undefined_source);
$targets = [];
$targets["outer"][] =& $undefined_source;
$undefined_source = "later";
echo $targets["outer"][0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|from-slot|later");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_append_target_rebinds_direct_alias_group() {
    let execution = run_source(
        r#"<?php
$value = "source";
$other =& $value;
$items["outer"][] =& $value;
$value = "from-value";
echo $items["outer"][0], "|", $other, "|";
$other = "from-other";
echo $items["outer"][0], "|", $value, "|";
$items["outer"][0] = "from-slot";
echo $value, "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-value|from-value|from-other|from-other|from-slot|from-slot"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_target_rebinds_array_offset_alias_group() {
    let execution = run_source(
        r#"<?php
$entry = "source";
$items["slot"] =& $entry;
$other =& $entry;
$copy["slot"] =& $entry;
$entry = "from-entry";
echo $items["slot"], "|", $copy["slot"], "|", $other, "|";
$copy["slot"] = "from-copy";
echo $entry, "|", $items["slot"], "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-entry|from-entry|from-entry|from-copy|from-copy|from-copy"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_globals_target_rebinds_array_offset_alias_group() {
    let execution = run_source(
        r#"<?php
$entry = "source";
$items["slot"] =& $entry;
$other =& $entry;
$GLOBALS["bag"]["slot"] =& $entry;
$entry = "from-entry";
echo $items["slot"], "|", $bag["slot"], "|", $other, "|";
$other = "from-other";
echo $items["slot"], "|", $GLOBALS["bag"]["slot"], "|", $entry, "|";
$GLOBALS["bag"]["slot"] = "from-global";
echo $entry, "|", $items["slot"], "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-entry|from-entry|from-entry|from-other|from-other|from-other|from-global|from-global|from-global"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_globals_root_target_rebinds_array_offset_alias_group() {
    let execution = run_source(
        r#"<?php
$entry = "source";
$items["slot"] =& $entry;
$other =& $entry;
$GLOBALS["target"] =& $entry;
echo $target, "|", $GLOBALS["target"], "|", $items["slot"], "|", $other, "|";
$target = "from-target";
echo $entry, "|", $items["slot"], "|", $GLOBALS["target"], "|", $other, "|";
$items["slot"] = "from-slot";
echo $target, "|", $GLOBALS["target"], "|", $entry, "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "source|source|source|source|from-target|from-target|from-target|from-target|from-slot|from-slot|from-slot|from-slot"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_target_rebinds_array_offset_alias_group() {
    let execution = run_source(
        r#"<?php
class Catalog {
    public $entries;
}
$catalog = new Catalog();
$entry = "source";
$items["slot"] =& $entry;
$other =& $entry;
$catalog->entries["slot"] =& $entry;
$entry = "from-entry";
echo $items["slot"], "|", $catalog->entries["slot"], "|", $other, "|";
$catalog->entries["slot"] = "from-property";
echo $entry, "|", $items["slot"], "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-entry|from-entry|from-entry|from-property|from-property|from-property"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn alias_graph_path_writes_sync_changed_slot_when_group_spans_same_root() {
    let execution = run_source(
        r#"<?php
$items = array("root" => array("slot" => "old"), "source" => "source-old");
$root =& $items["root"];
$source =& $items["source"];
$same =& $source;
$root["slot"] =& $source;
$source = "source-write";
echo $items["root"]["slot"], "|", $same, "|";
$items["root"]["slot"] = "slot-write";
echo $source, "|", $same, "|", $items["source"], "\n";

$_REQUEST["payload"] = array("root" => array("slot" => "old"), "source" => "request-old");
$requestRoot =& $_REQUEST["payload"]["root"];
$requestSource =& $_REQUEST["payload"]["source"];
$requestSame =& $requestSource;
$requestRoot["slot"] =& $requestSource;
$requestSource = "request-source";
echo $_REQUEST["payload"]["root"]["slot"], "|", $requestSame, "|";
$_REQUEST["payload"]["root"]["slot"] = "request-slot";
echo $requestSource, "|", $requestSame, "|", $_REQUEST["payload"]["source"], "\n";

class AliasGraphBox {
    public $items = array("root" => array("slot" => "old"), "source" => "box-old");
}
$box = new AliasGraphBox();
$boxRoot =& $box->items["root"];
$boxSource =& $box->items["source"];
$boxSame =& $boxSource;
$boxRoot["slot"] =& $boxSource;
$boxSource = "box-source";
echo $box->items["root"]["slot"], "|", $boxSame, "|";
$box->items["root"]["slot"] = "box-slot";
echo $boxSource, "|", $boxSame, "|", $box->items["source"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "source-write|source-write|slot-write|slot-write|slot-write\nrequest-source|request-source|request-slot|request-slot|request-slot\nbox-source|box-source|box-slot|box-slot|box-slot"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_rebind_detaches_old_array_slot_alias_group() {
    let execution = run_source(
        r#"<?php
$items = array("slot" => "old");
$old =& $items["slot"];
$same =& $old;
$new = "new";
$items["slot"] =& $new;
$new = "changed";
echo $items["slot"], "|", $old, "|", $same, "|", $new, "\n";
$same = "old-write";
echo $items["slot"], "|", $old, "|", $same, "|", $new, "\n";

class RebindDetachBox {
    public $items = array("slot" => "box-old");
}
$box = new RebindDetachBox();
$propertyOld =& $box->items["slot"];
$propertySame =& $propertyOld;
$propertyNew = "box-new";
$box->items["slot"] =& $propertyNew;
$propertyNew = "box-changed";
echo $box->items["slot"], "|", $propertyOld, "|", $propertySame, "|", $propertyNew, "\n";
$propertySame = "box-old-write";
echo $box->items["slot"], "|", $propertyOld, "|", $propertySame, "|", $propertyNew;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "changed|old|old|changed\nchanged|old-write|old-write|changed\nbox-changed|box-old|box-old|box-changed\nbox-changed|box-old-write|box-old-write|box-changed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_rebinds_non_direct_object_holder_slot_detaching_old_aliases() {
    let execution = run_source(
        r#"<?php
class NonDirectRebindBag {
    public $items = array("slot" => "old");
}

$holders = array("bag" => new NonDirectRebindBag(), "dynamic" => new NonDirectRebindBag());

$bag = $holders["bag"];
$old =& $bag->items["slot"];
$same =& $old;
$new = "new";
$holders["bag"]->items["slot"] =& $new;
$new = "changed";
echo "holder-rebind=", $bag->items["slot"], "|", $old, "|", $same, "|", $new, "\n";
$same = "old-write";
echo "holder-old=", $bag->items["slot"], "|", $old, "|", $same, "|", $new, "\n";

$property = "items";
$dynamicBag = $holders["dynamic"];
$dynamicOld =& $dynamicBag->items["slot"];
$dynamicSame =& $dynamicOld;
$dynamicNew = "dynamic-new";
$holders["dynamic"]->{$property}["slot"] =& $dynamicNew;
$dynamicNew = "dynamic-changed";
echo "dynamic-rebind=", $dynamicBag->items["slot"], "|", $dynamicOld, "|", $dynamicSame, "|", $dynamicNew, "\n";
$dynamicSame = "dynamic-old-write";
echo "dynamic-old=", $dynamicBag->items["slot"], "|", $dynamicOld, "|", $dynamicSame, "|", $dynamicNew;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "holder-rebind=changed|old|old|changed\nholder-old=changed|old-write|old-write|changed\ndynamic-rebind=dynamic-changed|old|old|dynamic-changed\ndynamic-old=dynamic-changed|dynamic-old-write|dynamic-old-write|dynamic-changed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_expression_root_object_property_array_source_aliases_slot() {
    let execution = run_source(
        r#"<?php
class ExpressionRootReferenceBox {
    public $items = array("slot" => "old");
    public $dynamic = array("slot" => "dyn-old");
}

function expression_root_reference_box() {
    static $box;
    if (!$box) {
        $box = new ExpressionRootReferenceBox();
    }
    return $box;
}

$box = expression_root_reference_box();
$alias =& expression_root_reference_box()->items["slot"];
$alias = "via-alias";
echo $box->items["slot"], "|";
$box->items["slot"] = "via-box";
echo $alias, "\n";

$property = "dynamic";
$dynamicAlias =& expression_root_reference_box()->{$property}["slot"];
$dynamicAlias = "dynamic-alias";
echo $box->dynamic["slot"], "|";
$box->dynamic["slot"] = "dynamic-box";
echo $dynamicAlias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "via-alias|via-box\ndynamic-alias|dynamic-box"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_variable_source_to_variable_executes_current_subset() {
    let execution = run_source(
        r#"<?php
function parse_args($args, $defaults) {
    if (is_array($args)) {
        $parsed_args =& $args;
    }
    return array_merge($defaults, $parsed_args);
}
$parsed = parse_args(["name" => "Ada"], ["role" => "admin"]);
echo $parsed["role"], "|", $parsed["name"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "admin|Ada");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_syntax_inside_unexecuted_function_body_is_registered() {
    let execution = run_source(
        r#"<?php
function parse_args($args) {
    $parsed_args =& $args;
    return $parsed_args;
}
echo "registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "registered");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_offset_source_inside_unexecuted_body_is_registered() {
    let execution = run_source(
        r#"<?php
function descend(&$items) {
    $cursor =& $items[0];
    return $cursor;
}
echo "registered";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "registered");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_existing_array_offset_source_aliases_direct_slot() {
    let execution = run_source(
        r#"<?php
$items = ["name" => "Ada"];
$key = "name";
$alias =& $items[$key];
$alias = "Grace";
echo $items["name"];
echo "|";
$items["name"] = "Katherine";
echo $alias;
unset($alias);
$items["name"] = "Dorothy";
echo "|";
echo $items["name"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace|Katherine|Dorothy");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_existing_array_offset_source_can_chain_direct_aliases() {
    let execution = run_source(
        r#"<?php
$items = [0 => "zero"];
$alias =& $items[0];
$other =& $alias;
$other = "one";
echo $items[0];
$items[0] = "two";
echo "|";
echo $alias;
echo "|";
echo $other;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "one|two|two");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_missing_array_offset_source_materializes_null_slot() {
    let execution = run_source(
        r#"<?php
$items = [];
$alias =& $items["missing"];
$alias = "materialized";
echo $items["missing"];
echo "|";
$items["missing"] = "updated";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "materialized|updated");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_offset_source_aliases_direct_slot() {
    let execution = run_source(
        r#"<?php
$items = ["outer" => ["inner" => "x"]];
$alias =& $items["outer"]["inner"];
$alias = "from-alias";
echo $items["outer"]["inner"];
echo "|";
$items["outer"]["inner"] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_offset_source_materializes_path() {
    let execution = run_source(
        r#"<?php
$items = [];
$alias =& $items["outer"]["inner"];
$alias = "created";
echo $items["outer"]["inner"];
echo "|";
$items["outer"]["inner"] = "updated";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "created|updated");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_undefined_and_null_array_offset_sources_materialize_array_roots() {
    let execution = run_source(
        r#"<?php
$undefined_alias =& $undefined["slot"];
$undefined_alias = "created";
echo $undefined["slot"];
echo "|";
$undefined["slot"] = "updated";
echo $undefined_alias;
echo "|";
$nullable = null;
$null_alias =& $nullable["slot"];
$null_alias = "from-null";
echo $nullable["slot"];
echo "|";
$nullable["slot"] = "changed";
echo $null_alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "created|updated|from-null|changed");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_array_offset_source_remains_boundary() {
    let error = runtime_error(
        r#"<?php
$items = 1;
$alias =& $items[0];
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot read offset on int"
    );
}

#[test]
fn reference_assignment_method_call_source_inside_unexecuted_body_is_registered() {
    let execution = run_source(
        r#"<?php
class Parser {
    public function make() {
        return 1;
    }

    public function register() {
        $entry =& $this->make();
        return "registered";
    }
}
echo "loaded";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "loaded");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_method_call_source_executes_as_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
class Parser {
    public function make() {
        return 1;
    }

    public function run() {
        $entry =& $this->make();
    }
}
$parser = new Parser();
$parser->run();
"#,
    );

    assert_eq!(error.line, 8);
    assert_eq!(error.column, 19);
    assert_eq!(
        error.message,
        "unsupported call make(): function does not return by reference"
    );
}

#[test]
fn reference_assignment_object_property_array_target_inside_unexecuted_body_is_registered() {
    let execution = run_source(
        r#"<?php
class Catalog {
    public $entries;

    public function register() {
        $entry = 1;
        $this->entries[$entry] =& $entry;
    }
}
echo "loaded";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "loaded");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_target_aliases_direct_variable_source() {
    let execution = run_source(
        r#"<?php
class Catalog {
    public $entries;

    public function run() {
        $key = "name";
        $entry = "Grace";
        $this->entries[$key] =& $entry;
        echo $this->entries["name"];
        echo "|";
        $entry = "Hedy";
        echo $this->entries["name"];
        echo "|";
        $this->entries["name"] = "Katherine";
        echo $entry;
        unset($entry);
        $entry = "detached";
        echo "|";
        echo $this->entries["name"], "|", $entry;
    }
}
$catalog = new Catalog();
$catalog->run();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace|Hedy|Katherine|Katherine|detached");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_target_materializes_null_property() {
    let execution = run_source(
        r#"<?php
class Catalog {
    public $entries;
}
$catalog = new Catalog();
$entry = "root";
$catalog->entries["slot"] =& $entry;
$entry = "changed";
echo $catalog->entries["slot"];
echo "|";
$catalog->entries["slot"] = "from-slot";
echo $entry;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_append_target_aliases_direct_variable_source() {
    let execution = run_source(
        r#"<?php
class Catalog {
    public $entries;
}
$catalog = new Catalog();
$entry = "Grace";
$catalog->entries[] =& $entry;
echo $catalog->entries[0];
echo "|";
$entry = "Hedy";
echo $catalog->entries[0];
echo "|";
$catalog->entries[0] = "Katherine";
echo $entry;
unset($entry);
$entry = "detached";
echo "|";
echo $catalog->entries[0], "|", $entry;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace|Hedy|Katherine|Katherine|detached");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_parameters_write_back_direct_object_property_array_arguments() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];

    public function tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
    }
}

function cache_mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
cache_mark($cache->cache["options"]["alloptions"], "function");
$cache->tag($cache->cache["options"]["alloptions"], "method");
echo $cache->cache["options"]["alloptions"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "cold:function:method");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn named_static_reference_parameters_write_back_direct_object_property_array_arguments() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
    }
}

$cache = new WP_Object_Cache();
$seen = "start";
Cache_Marker::tag($seen, "var");
$cache->cache["options"]["alloptions"] = "cold";
Cache_Marker::tag($cache->cache["options"]["alloptions"], "static");
echo $seen, "|", $cache->cache["options"]["alloptions"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "start:var|cold:static");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn self_and_static_reference_parameters_write_back_direct_object_property_array_arguments() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
    }

    public static function mark_self($cache) {
        self::tag($cache->cache["options"]["alloptions"], "self");
    }

    public static function mark_static($cache) {
        static::tag($cache->cache["options"]["alloptions"], "static");
    }
}

class Child_Cache_Marker extends Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ":child-" . $suffix;
    }
}

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
Cache_Marker::mark_self($cache);
Child_Cache_Marker::mark_static($cache);
echo $cache->cache["options"]["alloptions"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "cold:self:child-static");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parent_reference_parameters_write_back_direct_object_property_array_arguments() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];
}

class Parent_Cache_Marker {
    public function mark_parent(&$value, $suffix) {
        $value = $value . ":parent-" . $suffix;
    }

    public static function tag_parent(&$value, $suffix) {
        $value = $value . ":parent-static-" . $suffix;
    }
}

class Child_Cache_Marker extends Parent_Cache_Marker {
    public function mark($cache) {
        parent::mark_parent($cache->cache["options"]["alloptions"], "method");
        parent::tag_parent($cache->cache["options"]["alloptions"], "method");
    }
}

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$marker = new Child_Cache_Marker();
$marker->mark($cache);
echo $cache->cache["options"]["alloptions"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "cold:parent-method:parent-static-method");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_nested_array_append_target_aliases_direct_variable_source()
{
    let execution = run_source(
        r#"<?php
class Catalog {
    public $groups;
}
$catalog = new Catalog();
$entry = "Ada";
$catalog->groups["names"][] =& $entry;
echo $catalog->groups["names"][0];
echo "|";
$entry = "Lovelace";
echo $catalog->groups["names"][0];
echo "|";
$catalog->groups["names"][0] = "Byron";
echo $entry;
unset($entry);
$entry = "detached";
echo "|";
echo $catalog->groups["names"][0], "|", $entry;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada|Lovelace|Byron|Byron|detached");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_deeper_array_target_aliases_direct_variable_source() {
    let execution = run_source(
        r#"<?php
class Catalog {
    public $groups;
}
$catalog = new Catalog();
$entry = "main";
$catalog->groups["labels"]["primary"] =& $entry;
$entry = "changed";
echo $catalog->groups["labels"]["primary"];
echo "|";
$catalog->groups["labels"]["primary"] = "from-slot";
echo $entry;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_target_rebinds_direct_alias_group() {
    let execution = run_source(
        r#"<?php
class Catalog {
    public $entries;
}
$catalog = new Catalog();
$entry = "source";
$other =& $entry;
$catalog->entries["slot"] =& $entry;
$entry = "from-entry";
echo $catalog->entries["slot"], "|", $other, "|";
$other = "from-other";
echo $catalog->entries["slot"], "|", $entry, "|";
$catalog->entries["slot"] = "from-slot";
echo $entry, "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-entry|from-entry|from-other|from-other|from-slot|from-slot"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_append_target_rebinds_direct_alias_group() {
    let execution = run_source(
        r#"<?php
class Catalog {
    public $entries;
}
$catalog = new Catalog();
$entry = "source";
$other =& $entry;
$catalog->entries[] =& $entry;
$entry = "from-entry";
echo $catalog->entries[0], "|", $other, "|";
$other = "from-other";
echo $catalog->entries[0], "|", $entry, "|";
$catalog->entries[0] = "from-slot";
echo $entry, "|", $other;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "from-entry|from-entry|from-other|from-other|from-slot|from-slot"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_access_offset_target_reports_stable_boundary() {
    let error = runtime_error(
        r#"<?php
class Bag implements ArrayAccess {
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return null; }
    public function offsetSet($offset, $value) { }
    public function offsetUnset($offset) { }
}
$bag = new Bag();
$value = "Grace";
$bag["name"] =& $value;
"#,
    );

    assert_eq!(error.line, 10);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: ArrayAccess object offsets cannot be assigned by reference in the current runtime"
    );
}

#[test]
fn reference_assignment_array_access_reference_return_offset_target_reports_stable_boundary() {
    let error = runtime_error(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = [];
    public function offsetExists($offset) { return false; }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { }
    public function offsetUnset($offset) { }
}
$bag = new Bag();
$value = "Grace";
$bag["name"] =& $value;
"#,
    );

    assert_eq!(error.line, 11);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: ArrayAccess object offsets cannot be assigned by reference in the current runtime"
    );
}

#[test]
fn reference_assignment_object_property_array_access_target_reports_stable_boundary() {
    let error = runtime_error(
        r#"<?php
class Bag implements ArrayAccess {
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return null; }
    public function offsetSet($offset, $value) { }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
}
$holder = new Holder();
$holder->bag = new Bag();
$value = "Grace";
$holder->bag["name"] =& $value;
"#,
    );

    assert_eq!(error.line, 14);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: ArrayAccess object offsets cannot be assigned by reference in the current runtime"
    );
}

#[test]
fn reference_assignment_array_access_offset_source_binds_bounded_offset_get_root() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = [];
    public function offsetExists($offset) { return false; }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
$bag = new Bag();
$alias =& $bag["name"];
$alias = "Grace";
echo $bag["name"], "|", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Grace|Grace");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_access_offset_get_bucket_copy_preserves_nested_reference_slots() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = array();
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$target = "seed";
$bag = new Bag();
$bag->items["outer"] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$bag->items["outer"]["id"]["function"] =& $target;

$bucket = $bag["outer"];
foreach ($bucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "via-offset";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "plain-copy";
    }
}
unset($node);

echo $target, "|", $bag->items["outer"]["id"]["function"], "|", $bag->items["outer"]["id"]["accepted_args"], "|", $bag->items["outer"]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "via-offset|via-offset|1|plain");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_access_bucket_copy_helper_parameter_preserves_nested_reference_slots() {
    let execution = run_source(
        r#"<?php
class HookStore implements ArrayAccess {
    public $hook = array();
    public function offsetExists($offset) { return isset($this->hook[$offset]); }
    public function offsetGet($offset) { return $this->hook[$offset]; }
    public function offsetSet($offset, $value) { $this->hook[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->hook[$offset]); }
}

function helper($bucket) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = "via-helper";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = "plain-copy";
        }
    }
    unset($node);
}

$target = "seed";
$holder = new HookStore();
$holder->hook[10] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$holder->hook[10]["id"]["function"] =& $target;

$bucket = $holder[10];
helper($bucket);

echo $target, "|", $holder->hook[10]["id"]["function"], "|", $holder->hook[10]["id"]["accepted_args"], "|", $holder->hook[10]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "via-helper|via-helper|1|plain");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_access_bucket_copy_helper_parameter_replacement_detaches_reference_slots() {
    let execution = run_source(
        r#"<?php
class HookStore implements ArrayAccess {
    public $hook = array();
    public function offsetExists($offset) { return isset($this->hook[$offset]); }
    public function offsetGet($offset) { return $this->hook[$offset]; }
    public function offsetSet($offset, $value) { $this->hook[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->hook[$offset]); }
}

function helper($bucket) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = "first";
        }
    }
    unset($node);

    $bucket = array("id" => array("function" => "local", "accepted_args" => 9));
    $bucket["id"]["function"] = "reused";
}

$target = "seed";
$holder = new HookStore();
$holder->hook[10] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$holder->hook[10]["id"]["function"] =& $target;

$bucket = $holder[10];
helper($bucket);

echo $target, "|", $holder->hook[10]["id"]["function"], "|", $bucket["id"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|first|first");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_array_access_offset_set_bucket_arbitrary_reference_slots() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1681/arrayaccess_offsetset_bucket_arbitrary_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout"))
        .expect("read ArrayAccess offsetSet stored-bucket COW probe expectation");
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP ArrayAccess offsetSet stored-bucket COW probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn array_access_offset_set_bucket_copy_preserves_arbitrary_nested_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1681/arrayaccess_offsetset_bucket_arbitrary_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1681/arrayaccess_offsetset_bucket_arbitrary_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_array_access_append_offset_set_bucket_reference_slots() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1682/arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout"))
        .expect("read ArrayAccess append offsetSet stored-bucket COW probe expectation");
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP ArrayAccess append offsetSet stored-bucket COW probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_array_access_exact_append_offset_set_empty_key_reference_slots() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1682/arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout"))
        .expect("read exact ArrayAccess append offsetSet stored-bucket COW probe expectation");
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP exact ArrayAccess append offsetSet stored-bucket COW probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1682/arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1682/arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1682/arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1682/arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_property_held_array_access_append_offset_set_bucket_reference_slots() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1683/property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read property-held ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP property-held ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_property_held_array_access_exact_append_offset_set_empty_key_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1683/property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read exact property-held ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP exact property-held ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn property_held_array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1683/property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1683/property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn property_held_array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1683/property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1683/property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_dynamic_property_held_array_access_append_offset_set_bucket_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1684/dynamic_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read dynamic property-held ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP dynamic property-held ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_dynamic_property_held_array_access_exact_append_offset_set_empty_key_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1684/dynamic_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read exact dynamic property-held ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP exact dynamic property-held ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn dynamic_property_held_array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1684/dynamic_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1684/dynamic_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_property_held_array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1684/dynamic_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1684/dynamic_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_non_direct_holder_property_held_array_access_append_offset_set_bucket_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1685/non_direct_holder_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read non-direct holder property-held ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP non-direct holder property-held ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_non_direct_holder_property_held_array_access_exact_append_offset_set_empty_key_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1685/non_direct_holder_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read exact non-direct holder property-held ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP exact non-direct holder property-held ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn non_direct_holder_property_held_array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1685/non_direct_holder_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1685/non_direct_holder_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn non_direct_holder_property_held_array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1685/non_direct_holder_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1685/non_direct_holder_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_dynamic_non_direct_holder_property_held_array_access_append_offset_set_bucket_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1686/dynamic_non_direct_holder_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read dynamic non-direct holder property-held ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP dynamic non-direct holder property-held ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_dynamic_non_direct_holder_property_held_array_access_exact_append_offset_set_empty_key_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1686/dynamic_non_direct_holder_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read exact dynamic non-direct holder property-held ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP exact dynamic non-direct holder property-held ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn dynamic_non_direct_holder_property_held_array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1686/dynamic_non_direct_holder_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1686/dynamic_non_direct_holder_property_held_arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_non_direct_holder_property_held_array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1686/dynamic_non_direct_holder_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1686/dynamic_non_direct_holder_property_held_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_magic_property_array_access_append_offset_set_bucket_reference_slots() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1687/magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read magic-property ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP magic-property ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_magic_property_array_access_exact_append_offset_set_empty_key_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1687/magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read exact magic-property ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP exact magic-property ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn magic_property_array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1687/magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1687/magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_property_array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots()
{
    let source = include_str!(
        "../../tests/fixtures/milestone1687/magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1687/magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_dynamic_magic_property_array_access_append_offset_set_bucket_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1687/dynamic_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read dynamic magic-property ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP dynamic magic-property ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_dynamic_magic_property_array_access_exact_append_offset_set_empty_key_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1687/dynamic_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read exact dynamic magic-property ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP exact dynamic magic-property ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn dynamic_magic_property_array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1687/dynamic_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1687/dynamic_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_magic_property_array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1687/dynamic_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1687/dynamic_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_preserves_non_direct_magic_property_array_access_append_offset_set_bucket_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1688/non_direct_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read non-direct magic-property ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP non-direct magic-property ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_non_direct_magic_property_array_access_exact_append_offset_set_empty_key_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1688/non_direct_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read exact non-direct magic-property ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP exact non-direct magic-property ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_dynamic_non_direct_magic_property_array_access_append_offset_set_bucket_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1688/dynamic_non_direct_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read dynamic non-direct magic-property ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP dynamic non-direct magic-property ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_preserves_dynamic_non_direct_magic_property_array_access_exact_append_offset_set_empty_key_reference_slots(
) {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1688/dynamic_non_direct_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout")).expect(
        "read exact dynamic non-direct magic-property ArrayAccess append offsetSet stored-bucket COW probe expectation",
    );
    let output = Command::new("php").arg(&fixture).output().expect(
        "run system PHP exact dynamic non-direct magic-property ArrayAccess append offsetSet stored-bucket COW probe",
    );

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.trim_end_matches('\n')
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn non_direct_magic_property_array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1688/non_direct_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1688/non_direct_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn non_direct_magic_property_array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1688/non_direct_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1688/non_direct_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_non_direct_magic_property_array_access_append_offset_set_bucket_copy_preserves_nested_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1688/dynamic_non_direct_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1688/dynamic_non_direct_magic_property_arrayaccess_append_offsetset_bucket_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_non_direct_magic_property_array_access_exact_append_offset_set_bucket_copy_uses_empty_key_reference_slots(
) {
    let source = include_str!(
        "../../tests/fixtures/milestone1688/dynamic_non_direct_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1688/dynamic_non_direct_magic_property_arrayaccess_exact_append_offsetset_empty_key_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_mutates_by_reference_magic_get_plain_array_append_reference_slots() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1689/magic_property_plain_array_append_reference_slot_cow.php",
    );
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP magic-property plain-array append COW probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "via-magic-array-append|via-magic-array-label|via-magic-array-append|via-magic-array-label|1|plain|plain"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_mutates_dynamic_by_reference_magic_get_plain_array_append_reference_slots() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1689/dynamic_magic_property_plain_array_append_reference_slot_cow.php",
    );
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP dynamic magic-property plain-array append COW probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "via-dynamic-magic-array-append|via-dynamic-magic-array-label|via-dynamic-magic-array-append|via-dynamic-magic-array-label|1|plain|plain"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_by_value_magic_get_plain_array_append_notices_and_does_not_mutate() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1689/magic_property_plain_array_append_by_value_noop.php",
    );
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP by-value magic-property plain-array append no-op probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        concat!(
            "notice:Indirect modification of overloaded property ",
            "Milestone1689_ByValueMagicPlainArrayAppendBox::$missing has no effect\n",
            "no-op"
        )
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_mutates_non_direct_by_reference_magic_get_plain_array_append_reference_slots() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1689/non_direct_magic_property_plain_array_append_reference_slot_cow.php",
    );
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP non-direct magic-property plain-array append COW probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "via-non-direct-magic-array-append|via-non-direct-magic-array-label|via-non-direct-magic-array-append|via-non-direct-magic-array-label|1|plain|plain"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn system_php_mutates_dynamic_non_direct_by_reference_magic_get_plain_array_append_reference_slots()
{
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/fixtures/milestone1689/dynamic_non_direct_magic_property_plain_array_append_reference_slot_cow.php",
    );
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP dynamic non-direct magic-property plain-array append COW probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "via-dynamic-non-direct-magic-array-append|via-dynamic-non-direct-magic-array-label|via-dynamic-non-direct-magic-array-append|via-dynamic-non-direct-magic-array-label|1|plain|plain"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn magic_property_plain_array_append_preserves_nested_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1689/magic_property_plain_array_append_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1689/magic_property_plain_array_append_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_magic_property_plain_array_append_preserves_nested_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1689/dynamic_magic_property_plain_array_append_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1689/dynamic_magic_property_plain_array_append_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn by_value_magic_property_plain_array_append_notices_and_does_not_mutate() {
    let source = include_str!(
        "../../tests/fixtures/milestone1689/magic_property_plain_array_append_by_value_noop.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1689/magic_property_plain_array_append_by_value_noop.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn non_direct_magic_property_plain_array_append_preserves_nested_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1689/non_direct_magic_property_plain_array_append_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1689/non_direct_magic_property_plain_array_append_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_non_direct_magic_property_plain_array_append_preserves_nested_reference_slots() {
    let source = include_str!(
        "../../tests/fixtures/milestone1689/dynamic_non_direct_magic_property_plain_array_append_reference_slot_cow.php"
    );
    let expected = include_str!(
        "../../tests/fixtures/milestone1689/dynamic_non_direct_magic_property_plain_array_append_reference_slot_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected.trim_end_matches('\n'));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn system_php_mutates_by_reference_magic_get_plain_array_nested_append_reference_slots() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1690/magic_property_plain_array_nested_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1690/magic_property_plain_array_nested_append_reference_slot_cow.stdout",
    );
}

#[test]
fn system_php_mutates_dynamic_by_reference_magic_get_plain_array_nested_append_reference_slots() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1690/dynamic_magic_property_plain_array_nested_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1690/dynamic_magic_property_plain_array_nested_append_reference_slot_cow.stdout",
    );
}

#[test]
fn system_php_mutates_non_direct_by_reference_magic_get_plain_array_nested_append_reference_slots()
{
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1690/non_direct_magic_property_plain_array_nested_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1690/non_direct_magic_property_plain_array_nested_append_reference_slot_cow.stdout",
    );
}

#[test]
fn system_php_mutates_dynamic_non_direct_by_reference_magic_get_plain_array_nested_append_reference_slots(
) {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1690/dynamic_non_direct_magic_property_plain_array_nested_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1690/dynamic_non_direct_magic_property_plain_array_nested_append_reference_slot_cow.stdout",
    );
}

#[test]
fn system_php_by_value_magic_get_plain_array_nested_append_notices_and_does_not_mutate() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1690/magic_property_plain_array_nested_append_by_value_noop.php",
        "../tests/fixtures/milestone1690/magic_property_plain_array_nested_append_by_value_noop.stdout",
    );
}

#[test]
fn magic_property_plain_array_nested_append_preserves_nested_reference_slots() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1690/magic_property_plain_array_nested_append_reference_slot_cow.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1690/magic_property_plain_array_nested_append_reference_slot_cow.stdout"
        ),
    );
}

#[test]
fn dynamic_magic_property_plain_array_nested_append_preserves_nested_reference_slots() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1690/dynamic_magic_property_plain_array_nested_append_reference_slot_cow.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1690/dynamic_magic_property_plain_array_nested_append_reference_slot_cow.stdout"
        ),
    );
}

#[test]
fn non_direct_magic_property_plain_array_nested_append_preserves_nested_reference_slots() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1690/non_direct_magic_property_plain_array_nested_append_reference_slot_cow.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1690/non_direct_magic_property_plain_array_nested_append_reference_slot_cow.stdout"
        ),
    );
}

#[test]
fn dynamic_non_direct_magic_property_plain_array_nested_append_preserves_nested_reference_slots() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1690/dynamic_non_direct_magic_property_plain_array_nested_append_reference_slot_cow.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1690/dynamic_non_direct_magic_property_plain_array_nested_append_reference_slot_cow.stdout"
        ),
    );
}

#[test]
fn by_value_magic_property_plain_array_nested_append_notices_and_does_not_mutate() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1690/magic_property_plain_array_nested_append_by_value_noop.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1690/magic_property_plain_array_nested_append_by_value_noop.stdout"
        ),
    );
}

#[test]
fn system_php_mutates_by_reference_magic_get_plain_array_deep_nested_append_reference_slots() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1691/magic_property_plain_array_deep_nested_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1691/magic_property_plain_array_deep_nested_append_reference_slot_cow.stdout",
    );
}

#[test]
fn system_php_mutates_dynamic_by_reference_magic_get_plain_array_deep_nested_append_reference_slots(
) {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1691/dynamic_magic_property_plain_array_deep_nested_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1691/dynamic_magic_property_plain_array_deep_nested_append_reference_slot_cow.stdout",
    );
}

#[test]
fn system_php_mutates_non_direct_by_reference_magic_get_plain_array_deep_nested_append_reference_slots(
) {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1691/non_direct_magic_property_plain_array_deep_nested_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1691/non_direct_magic_property_plain_array_deep_nested_append_reference_slot_cow.stdout",
    );
}

#[test]
fn system_php_mutates_dynamic_non_direct_by_reference_magic_get_plain_array_deep_nested_append_reference_slots(
) {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1691/dynamic_non_direct_magic_property_plain_array_deep_nested_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1691/dynamic_non_direct_magic_property_plain_array_deep_nested_append_reference_slot_cow.stdout",
    );
}

#[test]
fn system_php_by_value_magic_get_plain_array_deep_nested_append_notices_and_does_not_mutate() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1691/magic_property_plain_array_deep_nested_append_by_value_noop.php",
        "../tests/fixtures/milestone1691/magic_property_plain_array_deep_nested_append_by_value_noop.stdout",
    );
}

#[test]
fn magic_property_plain_array_deep_nested_append_preserves_nested_reference_slots() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1691/magic_property_plain_array_deep_nested_append_reference_slot_cow.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1691/magic_property_plain_array_deep_nested_append_reference_slot_cow.stdout"
        ),
    );
}

#[test]
fn dynamic_magic_property_plain_array_deep_nested_append_preserves_nested_reference_slots() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1691/dynamic_magic_property_plain_array_deep_nested_append_reference_slot_cow.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1691/dynamic_magic_property_plain_array_deep_nested_append_reference_slot_cow.stdout"
        ),
    );
}

#[test]
fn non_direct_magic_property_plain_array_deep_nested_append_preserves_nested_reference_slots() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1691/non_direct_magic_property_plain_array_deep_nested_append_reference_slot_cow.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1691/non_direct_magic_property_plain_array_deep_nested_append_reference_slot_cow.stdout"
        ),
    );
}

#[test]
fn dynamic_non_direct_magic_property_plain_array_deep_nested_append_preserves_nested_reference_slots(
) {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1691/dynamic_non_direct_magic_property_plain_array_deep_nested_append_reference_slot_cow.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1691/dynamic_non_direct_magic_property_plain_array_deep_nested_append_reference_slot_cow.stdout"
        ),
    );
}

#[test]
fn by_value_magic_property_plain_array_deep_nested_append_notices_and_does_not_mutate() {
    assert_run_source_fixture_matches_stdout(
        include_str!(
            "../../tests/fixtures/milestone1691/magic_property_plain_array_deep_nested_append_by_value_noop.php"
        ),
        include_str!(
            "../../tests/fixtures/milestone1691/magic_property_plain_array_deep_nested_append_by_value_noop.stdout"
        ),
    );
}

#[test]
fn milestone1692_system_php_magic_property_array_access_nested_append_fixtures_match() {
    for (fixture, expected) in [
        (
            "../tests/fixtures/milestone1692/magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1692/magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1692/dynamic_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1692/dynamic_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1692/non_direct_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1692/non_direct_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1692/dynamic_non_direct_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1692/dynamic_non_direct_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1692/magic_property_arrayaccess_nested_append_by_value_offsetget_noop.php",
            "../tests/fixtures/milestone1692/magic_property_arrayaccess_nested_append_by_value_offsetget_noop.stdout",
        ),
    ] {
        assert_system_php_fixture_matches_stdout(fixture, expected);
    }
}

#[test]
fn milestone1692_magic_property_array_access_nested_append_fixtures_match_runtime() {
    for (fixture, expected) in [
        (
            "../tests/fixtures/milestone1692/magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1692/magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1692/dynamic_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1692/dynamic_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1692/non_direct_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1692/non_direct_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1692/dynamic_non_direct_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1692/dynamic_non_direct_magic_property_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1692/magic_property_arrayaccess_nested_append_by_value_offsetget_noop.php",
            "../tests/fixtures/milestone1692/magic_property_arrayaccess_nested_append_by_value_offsetget_noop.stdout",
        ),
    ] {
        assert_run_source_fixture_path_matches_stdout(fixture, expected);
    }
}

#[test]
fn milestone1693_system_php_magic_property_array_access_two_key_nested_append_fixtures_match() {
    for (fixture, expected) in [
        (
            "../tests/fixtures/milestone1693/magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1693/magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1693/dynamic_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1693/dynamic_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1693/non_direct_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1693/non_direct_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1693/dynamic_non_direct_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1693/dynamic_non_direct_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1693/magic_property_arrayaccess_two_key_nested_append_by_value_offsetget_noop.php",
            "../tests/fixtures/milestone1693/magic_property_arrayaccess_two_key_nested_append_by_value_offsetget_noop.stdout",
        ),
    ] {
        assert_system_php_fixture_matches_stdout(fixture, expected);
    }
}

#[test]
fn milestone1693_magic_property_array_access_two_key_nested_append_fixtures_match_runtime() {
    for (fixture, expected) in [
        (
            "../tests/fixtures/milestone1693/magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1693/magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1693/dynamic_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1693/dynamic_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1693/non_direct_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1693/non_direct_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1693/dynamic_non_direct_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.php",
            "../tests/fixtures/milestone1693/dynamic_non_direct_magic_property_arrayaccess_two_key_nested_append_offsetget_reference_slot_cow.stdout",
        ),
        (
            "../tests/fixtures/milestone1693/magic_property_arrayaccess_two_key_nested_append_by_value_offsetget_noop.php",
            "../tests/fixtures/milestone1693/magic_property_arrayaccess_two_key_nested_append_by_value_offsetget_noop.stdout",
        ),
    ] {
        assert_run_source_fixture_path_matches_stdout(fixture, expected);
    }
}

#[test]
fn milestone1694_system_php_magic_property_mixed_array_access_nested_append_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1694/magic_property_mixed_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
        "../tests/fixtures/milestone1694/magic_property_mixed_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1694_magic_property_mixed_array_access_nested_append_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1694/magic_property_mixed_arrayaccess_nested_append_offsetget_reference_slot_cow.php",
        "../tests/fixtures/milestone1694/magic_property_mixed_arrayaccess_nested_append_offsetget_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1695_system_php_magic_property_mixed_by_value_outer_array_access_append_fixture_matches(
) {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1695/magic_property_mixed_arrayaccess_by_value_outer_nested_append_offsetget_reference_slot_cow.php",
        "../tests/fixtures/milestone1695/magic_property_mixed_arrayaccess_by_value_outer_nested_append_offsetget_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1695_magic_property_mixed_by_value_outer_array_access_append_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1695/magic_property_mixed_arrayaccess_by_value_outer_nested_append_offsetget_reference_slot_cow.php",
        "../tests/fixtures/milestone1695/magic_property_mixed_arrayaccess_by_value_outer_nested_append_offsetget_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1696_system_php_magic_property_mixed_by_value_outer_reference_source_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1696/magic_property_mixed_arrayaccess_by_value_outer_reference_source_cow.php",
        "../tests/fixtures/milestone1696/magic_property_mixed_arrayaccess_by_value_outer_reference_source_cow.stdout",
    );
}

#[test]
fn milestone1696_magic_property_mixed_by_value_outer_reference_source_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1696/magic_property_mixed_arrayaccess_by_value_outer_reference_source_cow.php",
        "../tests/fixtures/milestone1696/magic_property_mixed_arrayaccess_by_value_outer_reference_source_cow.stdout",
    );
}

#[test]
fn milestone1697_system_php_magic_property_by_value_mixed_reference_source_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1697/magic_property_by_value_mixed_arrayaccess_reference_source_cow.php",
        "../tests/fixtures/milestone1697/magic_property_by_value_mixed_arrayaccess_reference_source_cow.stdout",
    );
}

#[test]
fn milestone1697_magic_property_by_value_mixed_reference_source_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1697/magic_property_by_value_mixed_arrayaccess_reference_source_cow.php",
        "../tests/fixtures/milestone1697/magic_property_by_value_mixed_arrayaccess_reference_source_cow.stdout",
    );
}

#[test]
fn milestone1698_system_php_magic_property_long_mixed_reference_source_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1698/magic_property_long_mixed_arrayaccess_reference_source_cow.php",
        "../tests/fixtures/milestone1698/magic_property_long_mixed_arrayaccess_reference_source_cow.stdout",
    );
}

#[test]
fn milestone1698_magic_property_long_mixed_reference_source_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1698/magic_property_long_mixed_arrayaccess_reference_source_cow.php",
        "../tests/fixtures/milestone1698/magic_property_long_mixed_arrayaccess_reference_source_cow.stdout",
    );
}

#[test]
fn milestone1699_system_php_factory_magic_arrayaccess_backing_write_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1699/factory_magic_arrayaccess_reference_source_backing_write_cow.php",
        "../tests/fixtures/milestone1699/factory_magic_arrayaccess_reference_source_backing_write_cow.stdout",
    );
}

#[test]
fn milestone1699_factory_magic_arrayaccess_backing_write_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1699/factory_magic_arrayaccess_reference_source_backing_write_cow.php",
        "../tests/fixtures/milestone1699/factory_magic_arrayaccess_reference_source_backing_write_cow.stdout",
    );
}

#[test]
fn milestone1700_system_php_magic_arrayaccess_alias_unset_detach_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1700/magic_arrayaccess_reference_source_alias_unset_detach_cow.php",
        "../tests/fixtures/milestone1700/magic_arrayaccess_reference_source_alias_unset_detach_cow.stdout",
    );
}

#[test]
fn milestone1700_magic_arrayaccess_alias_unset_detach_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1700/magic_arrayaccess_reference_source_alias_unset_detach_cow.php",
        "../tests/fixtures/milestone1700/magic_arrayaccess_reference_source_alias_unset_detach_cow.stdout",
    );
}

#[test]
fn milestone1701_system_php_magic_get_this_property_append_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1701/magic_get_this_property_array_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1701/magic_get_this_property_array_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1701_magic_get_this_property_append_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1701/magic_get_this_property_array_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1701/magic_get_this_property_array_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1702_system_php_magic_get_this_property_deep_append_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1702/magic_get_this_property_deep_array_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1702/magic_get_this_property_deep_array_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1702_magic_get_this_property_deep_append_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1702/magic_get_this_property_deep_array_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1702/magic_get_this_property_deep_array_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1703_system_php_magic_get_private_this_property_method_write_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1703/magic_get_private_this_property_method_write_reference_slot_cow.php",
        "../tests/fixtures/milestone1703/magic_get_private_this_property_method_write_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1703_magic_get_private_this_property_method_write_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1703/magic_get_private_this_property_method_write_reference_slot_cow.php",
        "../tests/fixtures/milestone1703/magic_get_private_this_property_method_write_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1704_system_php_magic_get_dynamic_private_this_property_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1704/magic_get_dynamic_private_this_property_reference_slot_cow.php",
        "../tests/fixtures/milestone1704/magic_get_dynamic_private_this_property_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1704_magic_get_dynamic_private_this_property_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1704/magic_get_dynamic_private_this_property_reference_slot_cow.php",
        "../tests/fixtures/milestone1704/magic_get_dynamic_private_this_property_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1705_system_php_magic_get_this_property_offset_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1705/magic_get_this_property_offset_reference_slot_cow.php",
        "../tests/fixtures/milestone1705/magic_get_this_property_offset_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1705_magic_get_this_property_offset_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1705/magic_get_this_property_offset_reference_slot_cow.php",
        "../tests/fixtures/milestone1705/magic_get_this_property_offset_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1706_system_php_magic_get_this_property_offset_suffix_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1706/magic_get_this_property_offset_literal_suffix_reference_slot_cow.php",
        "../tests/fixtures/milestone1706/magic_get_this_property_offset_literal_suffix_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1706_magic_get_this_property_offset_suffix_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1706/magic_get_this_property_offset_literal_suffix_reference_slot_cow.php",
        "../tests/fixtures/milestone1706/magic_get_this_property_offset_literal_suffix_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1707_system_php_magic_get_this_property_offset_prefix_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1707/magic_get_this_property_offset_literal_prefix_reference_slot_cow.php",
        "../tests/fixtures/milestone1707/magic_get_this_property_offset_literal_prefix_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1707_magic_get_this_property_offset_prefix_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1707/magic_get_this_property_offset_literal_prefix_reference_slot_cow.php",
        "../tests/fixtures/milestone1707/magic_get_this_property_offset_literal_prefix_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1708_system_php_magic_get_this_property_offset_reference_source_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1708/magic_get_this_property_offset_reference_source_alias_cow.php",
        "../tests/fixtures/milestone1708/magic_get_this_property_offset_reference_source_alias_cow.stdout",
    );
}

#[test]
fn milestone1708_magic_get_this_property_offset_reference_source_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1708/magic_get_this_property_offset_reference_source_alias_cow.php",
        "../tests/fixtures/milestone1708/magic_get_this_property_offset_reference_source_alias_cow.stdout",
    );
}

#[test]
fn milestone1709_system_php_magic_get_nondirect_dynamic_offset_reference_source_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1709/magic_get_this_property_offset_nondirect_dynamic_reference_source_alias_cow.php",
        "../tests/fixtures/milestone1709/magic_get_this_property_offset_nondirect_dynamic_reference_source_alias_cow.stdout",
    );
}

#[test]
fn milestone1709_magic_get_nondirect_dynamic_offset_reference_source_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1709/magic_get_this_property_offset_nondirect_dynamic_reference_source_alias_cow.php",
        "../tests/fixtures/milestone1709/magic_get_this_property_offset_nondirect_dynamic_reference_source_alias_cow.stdout",
    );
}

#[test]
fn milestone1710_system_php_arrayaccess_literal_bucket_reference_source_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1710/arrayaccess_offsetget_literal_bucket_reference_source_cow.php",
        "../tests/fixtures/milestone1710/arrayaccess_offsetget_literal_bucket_reference_source_cow.stdout",
    );
}

#[test]
fn milestone1710_arrayaccess_literal_bucket_reference_source_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1710/arrayaccess_offsetget_literal_bucket_reference_source_cow.php",
        "../tests/fixtures/milestone1710/arrayaccess_offsetget_literal_bucket_reference_source_cow.stdout",
    );
}

#[test]
fn milestone1711_system_php_arrayaccess_literal_bucket_offsetset_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1711/arrayaccess_offsetset_literal_bucket_reference_slot_cow.php",
        "../tests/fixtures/milestone1711/arrayaccess_offsetset_literal_bucket_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1711_arrayaccess_literal_bucket_offsetset_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1711/arrayaccess_offsetset_literal_bucket_reference_slot_cow.php",
        "../tests/fixtures/milestone1711/arrayaccess_offsetset_literal_bucket_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1712_system_php_arrayaccess_literal_prefix_append_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1712/arrayaccess_offsetset_literal_prefix_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1712/arrayaccess_offsetset_literal_prefix_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1712_arrayaccess_literal_prefix_append_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1712/arrayaccess_offsetset_literal_prefix_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1712/arrayaccess_offsetset_literal_prefix_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1713_system_php_arrayaccess_literal_suffix_append_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1713/arrayaccess_offsetset_literal_suffix_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1713/arrayaccess_offsetset_literal_suffix_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1713_arrayaccess_literal_suffix_append_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1713/arrayaccess_offsetset_literal_suffix_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1713/arrayaccess_offsetset_literal_suffix_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1714_system_php_arrayaccess_if_else_append_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1714/arrayaccess_offsetset_if_else_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1714/arrayaccess_offsetset_if_else_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1714_arrayaccess_if_else_append_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1714/arrayaccess_offsetset_if_else_append_reference_slot_cow.php",
        "../tests/fixtures/milestone1714/arrayaccess_offsetset_if_else_append_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1715_system_php_arrayaccess_repeated_offset_parameter_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1715/arrayaccess_repeated_offset_parameter_reference_slot_cow.php",
        "../tests/fixtures/milestone1715/arrayaccess_repeated_offset_parameter_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1715_arrayaccess_repeated_offset_parameter_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1715/arrayaccess_repeated_offset_parameter_reference_slot_cow.php",
        "../tests/fixtures/milestone1715/arrayaccess_repeated_offset_parameter_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1716_system_php_arrayaccess_if_else_keyed_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1716/arrayaccess_offsetset_if_else_keyed_reference_slot_cow.php",
        "../tests/fixtures/milestone1716/arrayaccess_offsetset_if_else_keyed_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1716_arrayaccess_if_else_keyed_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1716/arrayaccess_offsetset_if_else_keyed_reference_slot_cow.php",
        "../tests/fixtures/milestone1716/arrayaccess_offsetset_if_else_keyed_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1717_system_php_property_held_arrayaccess_keyed_fixture_matches() {
    assert_system_php_fixture_matches_stdout(
        "../tests/fixtures/milestone1717/property_held_arrayaccess_keyed_reference_slot_cow.php",
        "../tests/fixtures/milestone1717/property_held_arrayaccess_keyed_reference_slot_cow.stdout",
    );
}

#[test]
fn milestone1717_property_held_arrayaccess_keyed_fixture_matches_runtime() {
    assert_run_source_fixture_path_matches_stdout(
        "../tests/fixtures/milestone1717/property_held_arrayaccess_keyed_reference_slot_cow.php",
        "../tests/fixtures/milestone1717/property_held_arrayaccess_keyed_reference_slot_cow.stdout",
    );
}

#[test]
fn property_held_array_access_bucket_copy_preserves_nested_reference_slots() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = array();
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $bag;
}

$target = "seed";
$bag = new Bag();
$bag->items["outer"] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$bag->items["outer"]["id"]["function"] =& $target;

$holder = new Holder();
$holder->bag = $bag;

$bucket = $holder->bag["outer"];
foreach ($bucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "via-holder";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "plain-copy";
    }
}
unset($node);

echo $target, "|", $bag->items["outer"]["id"]["function"], "|", $bag->items["outer"]["id"]["accepted_args"], "|", $bag->items["outer"]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "via-holder|via-holder|1|plain");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_property_held_array_access_bucket_copy_preserves_nested_reference_slots() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = array();
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $dynamicBag;
}

$target = "seed";
$bag = new Bag();
$bag->items["outer"] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$bag->items["outer"]["id"]["function"] =& $target;

$holder = new Holder();
$holder->dynamicBag = $bag;

$property = "dynamicBag";
$bucket = $holder->{$property}["outer"];
foreach ($bucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "via-dynamic";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "plain-copy";
    }
}
unset($node);

echo $target, "|", $bag->items["outer"]["id"]["function"], "|", $bag->items["outer"]["id"]["accepted_args"], "|", $bag->items["outer"]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "via-dynamic|via-dynamic|1|plain");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_parameter_preserves_property_held_array_access_bucket_copy_reference_slots() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = array();
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $hook;
}

function mutate_bucket($bucket) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = "via-helper";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = "plain-helper";
        }
    }
    unset($node);
}

$target = "seed";
$bag = new Bag();
$bag->items[10] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$bag->items[10]["id"]["function"] =& $target;

$holder = new Holder();
$holder->hook = $bag;
$bucket = $holder->hook[10];
mutate_bucket($bucket);

echo $target, "|", $bag->items[10]["id"]["function"], "|", $bag->items[10]["id"]["accepted_args"], "|", $bag->items[10]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "via-helper|via-helper|1|plain");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn function_parameter_preserves_dynamic_property_held_array_access_bucket_copy_reference_slots() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = array();
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $dynamicHook;
}

function mutate_dynamic_bucket($bucket) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = "via-dynamic-helper";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = "plain-helper";
        }
    }
    unset($node);
}

$target = "seed";
$bag = new Bag();
$bag->items[10] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$bag->items[10]["id"]["function"] =& $target;

$holder = new Holder();
$holder->dynamicHook = $bag;
$property = "dynamicHook";
$bucket = $holder->{$property}[10];
mutate_dynamic_bucket($bucket);

echo $target, "|", $bag->items[10]["id"]["function"], "|", $bag->items[10]["id"]["accepted_args"], "|", $bag->items[10]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "via-dynamic-helper|via-dynamic-helper|1|plain"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn non_direct_holder_array_access_bucket_copy_preserves_nested_reference_slots() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = array();
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $bag;
    public function __construct($bag) {
        $this->bag = $bag;
    }
}

class Registry {
    public $holder;
    public function holder() {
        return $this->holder;
    }
}

function make_holder($bag) {
    return new Holder($bag);
}

$key = "outer";
$arrayTarget = "array-seed";
$arrayBag = new Bag();
$arrayBag->items["outer"] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$arrayBag->items["outer"]["id"]["function"] =& $arrayTarget;
$holders = array("box" => new Holder($arrayBag));
$bucket = $holders["box"]->bag[$key];
foreach ($bucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "via-array-holder";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "plain-copy";
    }
}
unset($node);
echo $arrayTarget, "|", $arrayBag->items["outer"]["id"]["function"], "|", $arrayBag->items["outer"]["id"]["accepted_args"], "|", $arrayBag->items["outer"]["plain"]["function"], "\n";

$methodTarget = "method-seed";
$methodBag = new Bag();
$methodBag->items["outer"] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$methodBag->items["outer"]["id"]["function"] =& $methodTarget;
$registry = new Registry();
$registry->holder = new Holder($methodBag);
$bucket = $registry->holder()->bag[$key];
foreach ($bucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "via-method-holder";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "plain-copy";
    }
}
unset($node);
echo $methodTarget, "|", $methodBag->items["outer"]["id"]["function"], "|", $methodBag->items["outer"]["id"]["accepted_args"], "|", $methodBag->items["outer"]["plain"]["function"], "\n";

$exprTarget = "expr-seed";
$exprBag = new Bag();
$exprBag->items["outer"] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
    "plain" => array("function" => "plain", "accepted_args" => 1),
);
$exprBag->items["outer"]["id"]["function"] =& $exprTarget;
$bucket = make_holder($exprBag)->bag[$key];
foreach ($bucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "via-expression-holder";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "plain-copy";
    }
}
unset($node);
echo $exprTarget, "|", $exprBag->items["outer"]["id"]["function"], "|", $exprBag->items["outer"]["id"]["accepted_args"], "|", $exprBag->items["outer"]["plain"]["function"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "via-array-holder|via-array-holder|1|plain\nvia-method-holder|via-method-holder|1|plain\nvia-expression-holder|via-expression-holder|1|plain"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_access_nested_offset_source_binds_bounded_offset_get_root() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    private $items = ["outer" => ["slot" => "seed"]];
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}
$bag = new Bag();
$alias =& $bag["outer"]["slot"];
$alias = $alias . ":alias";
echo $bag["outer"]["slot"], "|", $alias, "\n";
$missing =& $bag["created"]["leaf"];
$missing = "made";
echo $bag["created"]["leaf"], "|", $missing;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed:alias|seed:alias\nmade|made");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_access_offset_source_by_value_detaches_with_notice() {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["name" => "seed"];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
$bag = new Bag();
$key = "name";
$alias =& $bag[$key];
$alias = "changed";
echo $alias, "|", $bag->items[$key];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|seed"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_access_nested_offset_source_by_value_detaches_with_notice() {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["outer" => ["slot" => "seed"]];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
$bag = new Bag();
$key = "outer";
$alias =& $bag[$key]["slot"];
$alias = "changed";
echo $alias, "|", $bag->items[$key]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|seed"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_property_held_array_access_offset_source_by_value_detaches_with_notice() {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["name" => "seed"];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
}
$holder = new Holder();
$holder->bag = new Bag();
$key = "name";
$alias =& $holder->bag[$key];
$alias = "changed";
echo $alias, "|", $holder->bag->items[$key];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|seed"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_property_held_array_access_nested_offset_source_by_value_detaches_with_notice(
) {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["outer" => ["slot" => "seed"]];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
}
$holder = new Holder();
$holder->bag = new Bag();
$key = "outer";
$alias =& $holder->bag[$key]["slot"];
$alias = "changed";
echo $alias, "|", $holder->bag->items[$key]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|seed"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_dynamic_property_held_array_access_nested_source_by_value_detaches_with_notice(
) {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["outer" => ["slot" => "seed"]];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
class Holder {
    public $dynamicBag;
}
$holder = new Holder();
$holder->dynamicBag = new Bag();
$property = "dynamicBag";
$alias =& $holder->{$property}["outer"]["slot"];
$alias = "changed";
echo $alias, "|", $holder->dynamicBag->items["outer"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|seed"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_access_append_source_by_value_detaches_with_notice() {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["" => "empty"];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}

$bag = new Bag();
$alias =& $bag[];
$alias = "changed";
echo $alias, "|", $bag->items[""];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|empty"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_property_held_array_access_append_source_by_value_detaches_with_notice() {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["" => "empty"];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
    public $dynamicBag;
}

$holder = new Holder();
$holder->bag = new Bag();
$alias =& $holder->bag[];
$alias = "changed";
echo $alias, "|", $holder->bag->items[""], "\n";

$holder->dynamicBag = new Bag();
$property = "dynamicBag";
$dynamic =& $holder->{$property}[];
$dynamic = "dynamic-changed";
echo $dynamic, "|", $holder->dynamicBag->items[""];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|empty\nnotice:Indirect modification of overloaded element of Bag has no effect\ndynamic-changed|empty"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_access_append_source_by_value_detaches_existing_target_alias() {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["" => "empty"];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}

$items = ["slot" => "old"];
$target =& $items["slot"];
$bag = new Bag();
$target =& $bag[];
$target = "changed";
echo $target, "|", $items["slot"], "|", $bag->items[""];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|old|empty"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_direct_holder_array_access_source_by_value_detaches_with_notice() {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["name" => "seed", "outer" => ["slot" => "nested"]];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
    public $dynamicBag;
    public function __construct($bag) {
        $this->bag = $bag;
        $this->dynamicBag = $bag;
    }
}
class Registry {
    public $holder;
    public function holder() {
        return $this->holder;
    }
}
function make_holder($bag) {
    return new Holder($bag);
}

$bag = new Bag();
$holders = ["box" => new Holder($bag)];
$key = "name";
$alias =& $holders["box"]->bag[$key];
$alias = "changed";
echo $alias, "|", $bag->items[$key], "\n";

$property = "dynamicBag";
$dynamic =& $holders["box"]->{$property}["outer"]["slot"];
$dynamic = "dynamic-changed";
echo $dynamic, "|", $bag->items["outer"]["slot"], "\n";

$registry = new Registry();
$registry->holder = new Holder($bag);
$method =& $registry->holder()->bag["outer"]["slot"];
$method = "method-changed";
echo $method, "|", $bag->items["outer"]["slot"], "\n";

$expr =& make_holder($bag)->bag["outer"]["slot"];
$expr = "expr-changed";
echo $expr, "|", $bag->items["outer"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|seed\nnotice:Indirect modification of overloaded element of Bag has no effect\ndynamic-changed|nested\nnotice:Indirect modification of overloaded element of Bag has no effect\nmethod-changed|nested\nnotice:Indirect modification of overloaded element of Bag has no effect\nexpr-changed|nested"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_direct_holder_array_access_append_source_by_value_detaches_with_notice()
{
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["" => "empty"];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
    public $dynamicBag;
    public function __construct($bag) {
        $this->bag = $bag;
        $this->dynamicBag = $bag;
    }
}
class Registry {
    public $holder;
    public function holder() {
        return $this->holder;
    }
}
function make_holder($bag) {
    return new Holder($bag);
}

$bag = new Bag();
$holders = ["box" => new Holder($bag)];
$alias =& $holders["box"]->bag[];
$alias = "changed";
echo $alias, "|", $bag->items[""], "\n";

$property = "dynamicBag";
$dynamic =& $holders["box"]->{$property}[];
$dynamic = "dynamic-changed";
echo $dynamic, "|", $bag->items[""], "\n";

$registry = new Registry();
$registry->holder = new Holder($bag);
$method =& $registry->holder()->bag[];
$method = "method-changed";
echo $method, "|", $bag->items[""], "\n";

$expr =& make_holder($bag)->bag[];
$expr = "expr-changed";
echo $expr, "|", $bag->items[""];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:Indirect modification of overloaded element of Bag has no effect\nchanged|empty\nnotice:Indirect modification of overloaded element of Bag has no effect\ndynamic-changed|empty\nnotice:Indirect modification of overloaded element of Bag has no effect\nmethod-changed|empty\nnotice:Indirect modification of overloaded element of Bag has no effect\nexpr-changed|empty"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_direct_holder_array_access_reference_source_still_binds() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = ["slot" => "seed"];
    public function offsetExists($offset) { return false; }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
    public function __construct($bag) {
        $this->bag = $bag;
    }
}
$bag = new Bag();
$holders = ["box" => new Holder($bag)];
$alias =& $holders["box"]->bag["slot"];
$alias = "changed";
echo $alias, "|", $bag->items["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_direct_holder_array_access_append_source_by_reference_still_aliases() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = ["" => "empty"];
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}
class Holder {
    public $bag;
    public function __construct($bag) {
        $this->bag = $bag;
    }
}
$bag = new Bag();
$holders = ["box" => new Holder($bag)];
$alias =& $holders["box"]->bag[];
$alias = "changed";
echo $alias, "|", $bag->items[""];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_direct_holder_plain_property_array_source_still_binds() {
    let execution = run_source(
        r#"<?php
class Holder {
    public $items = ["slot" => "seed"];
}
$holders = ["box" => new Holder()];
$alias =& $holders["box"]->items["slot"];
$alias = "changed";
echo $alias, "|", $holders["box"]->items["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_direct_holder_plain_property_array_append_source_still_aliases() {
    let execution = run_source(
        r#"<?php
class Holder {
    public $items = ["" => "empty"];
}
$holders = ["box" => new Holder()];
$alias =& $holders["box"]->items[];
$alias = "changed";
echo $alias, "|", $holders["box"]->items[0], "|", $holders["box"]->items[""];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed|empty");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_direct_holder_array_access_source_by_reference_still_aliases() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = ["outer" => ["slot" => "seed"]];
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}
class Holder {
    public $bag;
    public function __construct($bag) { $this->bag = $bag; }
}

$bag = new Bag();
$holders = ["box" => new Holder($bag)];
$alias =& $holders["box"]->bag["outer"]["slot"];
$alias = "changed";
echo $bag->items["outer"]["slot"], "|", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_direct_holder_plain_property_array_source_still_aliases() {
    let execution = run_source(
        r#"<?php
class Holder {
    public $items = ["outer" => ["slot" => "seed"]];
}

$holder = new Holder();
$holders = ["box" => $holder];
$alias =& $holders["box"]->items["outer"]["slot"];
$alias = "changed";
echo $holder->items["outer"]["slot"], "|", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_access_source_binds_bounded_offset_get_root() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    private $items = ["outer" => ["slot" => "seed"]];
    public function offsetExists($offset) { return false; }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { }
    public function offsetUnset($offset) { }
}
class Holder {
    public $bag;
}
$holder = new Holder();
$holder->bag = new Bag();
$alias =& $holder->bag["outer"]["slot"];
$alias = $alias . ":alias";
echo $holder->bag["outer"]["slot"], "|", $alias, "\n";
$missing =& $holder->bag["created"]["leaf"];
$missing = "made";
echo $holder->bag["created"]["leaf"], "|", $missing;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed:alias|seed:alias\nmade|made");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn property_held_array_access_reference_source_survives_holder_property_rebind() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items;
    public function __construct($items) { $this->items = $items; }
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}
class Holder {
    public $bag;
}
$holder = new Holder();
$holder->bag = new Bag(["slot" => "old"]);
$old = $holder->bag;
$alias =& $holder->bag["slot"];
$alias = "alias";
echo $old["slot"], "|", $holder->bag["slot"], "|", $alias, "\n";
$holder->bag = new Bag(["slot" => "new"]);
echo $old["slot"], "|", $holder->bag["slot"], "|", $alias, "\n";
$alias = "after";
echo $old["slot"], "|", $holder->bag["slot"], "|", $alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "alias|alias|alias\nalias|new|alias\nafter|new|after"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_parameter_accepts_non_direct_holder_array_access_source() {
    let execution = run_source(
        r#"<?php
class Bag implements ArrayAccess {
    public $items = ["outer" => ["slot" => "seed"]];
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

class Holder {
    public $bag;
    public $dynamicBag;
}

function touch_slot(&$value) {
    $value = $value . ":touched";
}

$holders = [];
$primary = new Holder();
$primary->bag = new Bag();
$holders["primary"] = $primary;
touch_slot($holders["primary"]->bag["outer"]["slot"]);
echo $primary->bag["outer"]["slot"], "\n";

$dynamic = new Holder();
$dynamic->dynamicBag = new Bag();
$holders["dynamic"] = $dynamic;
$property = "dynamicBag";
touch_slot($holders["dynamic"]->{$property}["outer"]["slot"]);
echo $dynamic->dynamicBag["outer"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed:touched\nseed:touched");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_parameter_accepts_non_direct_holder_plain_property_array_source() {
    let execution = run_source(
        r#"<?php
class Bag {
    public $items = ["outer" => ["slot" => "seed"]];
    public $dynamicItems = ["outer" => ["slot" => "dynamic"]];
}

function touch_slot(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

$holders = [];
$primary = new Bag();
$holders["primary"] = $primary;
touch_slot($holders["primary"]->items["outer"]["slot"], "named");
echo $primary->items["outer"]["slot"], "\n";

$dynamic = new Bag();
$holders["dynamic"] = $dynamic;
$property = "dynamicItems";
touch_slot($holders["dynamic"]->{$property}["outer"]["slot"], "selected");
echo $dynamic->dynamicItems["outer"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed:named\ndynamic:selected");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_parameter_accepts_magic_get_direct_variable_source() {
    let execution = run_source(
        r#"<?php
$storage = "initial";
$dynamicStorage = "dynamic";

class MagicBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

class DynamicMagicBox {
    public function &__get($name) {
        global $dynamicStorage;
        return $dynamicStorage;
    }
}

function touch_magic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

$box = new MagicBox();
touch_magic($box->missing, "plain");
echo $storage, "\n";

$dynamicBox = new DynamicMagicBox();
$property = "dynamic";
touch_magic($dynamicBox->{$property}, "selected");
echo $dynamicStorage;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "initial:plain\ndynamic:selected");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_parameter_accepts_magic_get_array_offset_source() {
    let execution = run_source(
        r#"<?php
$storage = ["slot" => "initial", "nested" => ["leaf" => "inside"]];
$dynamicStorage = [];

class MagicArrayBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

class DynamicMagicArrayBox {
    public function &__get($name) {
        global $dynamicStorage;
        return $dynamicStorage;
    }
}

function touch_magic_slot(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
}

$box = new MagicArrayBox();
touch_magic_slot($box->missing["slot"], "plain");
touch_magic_slot($box->missing["nested"]["leaf"], "nested");
echo $storage["slot"], "\n", $storage["nested"]["leaf"], "\n";

$dynamicBox = new DynamicMagicArrayBox();
$property = "dynamic";
touch_magic_slot($dynamicBox->{$property}["created"], "selected");
echo $dynamicStorage["created"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "initial:plain\ninside:nested\nnull:selected"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_get_array_access_reference_chain_reaches_backing_storage() {
    let execution = run_source(
        r#"<?php
class MagicArrayAccessBag implements ArrayAccess {
    private $storage = ["slot" => "seed"];

    public function offsetExists($offset) { return isset($this->storage[$offset]); }
    public function &offsetGet($offset) { return $this->storage[$offset]; }
    public function offsetSet($offset, $value) { $this->storage[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->storage[$offset]); }
    public function read($offset) { return $this->storage[$offset]; }
}

$bag = new MagicArrayAccessBag();

class MagicArrayAccessBox {
    public function &__get($name) {
        global $bag;
        return $bag;
    }
}

function touch_magic_array_access(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
}

$box = new MagicArrayAccessBox();
touch_magic_array_access($box->missing["slot"], "arg");

$alias =& $box->missing["created"];
$alias = "via-alias";

echo $bag->read("slot"), "\n", $bag->read("created"), "|", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed:arg\nvia-alias|via-alias");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_get_nested_array_access_reference_chain_reaches_inner_storage() {
    let execution = run_source(
        r#"<?php
class MagicNestedArrayAccessBag implements ArrayAccess {
    private $storage;

    public function __construct($storage = []) {
        $this->storage = $storage;
    }

    public function offsetExists($offset) { return isset($this->storage[$offset]); }
    public function &offsetGet($offset) { return $this->storage[$offset]; }
    public function offsetSet($offset, $value) { $this->storage[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->storage[$offset]); }
    public function read($offset) { return $this->storage[$offset]; }
}

$inner = new MagicNestedArrayAccessBag(["slot" => "seed"]);
$outer = new MagicNestedArrayAccessBag(["inner" => $inner]);

class MagicNestedArrayAccessBox {
    public function &__get($name) {
        global $outer;
        return $outer;
    }
}

function touch_magic_nested_array_access(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
}

$box = new MagicNestedArrayAccessBox();
touch_magic_nested_array_access($box->missing["inner"]["slot"], "arg");

$alias =& $box->missing["inner"]["created"];
$alias = "via-alias";

echo $inner->read("slot"), "\n", $inner->read("created"), "|", $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed:arg\nvia-alias|via-alias");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_magic_get_array_access_source_by_value_detaches_with_notice() {
    let execution = run_source(
        r#"<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class MagicByValueBag implements ArrayAccess {
    public $items = ["name" => "seed", "outer" => ["slot" => "nested"], "" => "empty"];
    public function offsetExists($offset) { return false; }
    public function offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { }
}

$valueBag = new MagicByValueBag();
$refBag = new MagicByValueBag();

class MagicObjectByValueGetBox {
    public function __get($name) {
        global $valueBag;
        return $valueBag;
    }
}

class MagicObjectByReferenceGetBox {
    public function &__get($name) {
        global $refBag;
        return $refBag;
    }
}

$box = new MagicObjectByValueGetBox();
$key = "name";
$alias =& $box->missing[$key];
$alias = "changed";
echo "value-offset:", $alias, "|", $valueBag->items[$key], "\n";

$nested =& $box->missing["outer"]["slot"];
$nested = "nested-changed";
echo "value-nested:", $nested, "|", $valueBag->items["outer"]["slot"], "\n";

$property = "dynamicMissing";
$dynamic =& $box->{$property}[$key];
$dynamic = "dynamic-changed";
echo "value-dynamic:", $dynamic, "|", $valueBag->items[$key], "\n";

$append =& $box->missing[];
$append = "append-changed";
echo "value-append:", $append, "|", $valueBag->items[""], "\n";

$refBox = new MagicObjectByReferenceGetBox();
$refAlias =& $refBox->missing[$key];
$refAlias = "ref-changed";
echo "ref-offset:", $refAlias, "|", $refBag->items[$key], "\n";

$refNested =& $refBox->missing["outer"]["slot"];
$refNested = "ref-nested-changed";
echo "ref-nested:", $refNested, "|", $refBag->items["outer"]["slot"], "\n";

$refProperty = "dynamicMissing";
$refDynamic =& $refBox->{$refProperty}[$key];
$refDynamic = "ref-dynamic-changed";
echo "ref-dynamic:", $refDynamic, "|", $refBag->items[$key], "\n";

$refAppend =& $refBox->missing[];
$refAppend = "ref-append-changed";
echo "ref-append:", $refAppend, "|", $refBag->items[""];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "notice:Indirect modification of overloaded element of MagicByValueBag has no effect\n",
            "value-offset:changed|seed\n",
            "notice:Indirect modification of overloaded element of MagicByValueBag has no effect\n",
            "value-nested:nested-changed|nested\n",
            "notice:Indirect modification of overloaded element of MagicByValueBag has no effect\n",
            "value-dynamic:dynamic-changed|seed\n",
            "notice:Indirect modification of overloaded element of MagicByValueBag has no effect\n",
            "value-append:append-changed|empty\n",
            "notice:Indirect modification of overloaded element of MagicByValueBag has no effect\n",
            "ref-offset:ref-changed|seed\n",
            "notice:Indirect modification of overloaded element of MagicByValueBag has no effect\n",
            "ref-nested:ref-nested-changed|nested\n",
            "notice:Indirect modification of overloaded element of MagicByValueBag has no effect\n",
            "ref-dynamic:ref-dynamic-changed|seed\n",
            "notice:Indirect modification of overloaded element of MagicByValueBag has no effect\n",
            "ref-append:ref-append-changed|empty"
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_magic_get_reference_source_fallbacks_still_alias() {
    let execution = run_source(
        r#"<?php
class MagicReferenceBag implements ArrayAccess {
    public $items = ["slot" => "seed", "" => "empty"];
    public function offsetExists($offset) { return isset($this->items[$offset]); }
    public function &offsetGet($offset) { return $this->items[$offset]; }
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    public function offsetUnset($offset) { unset($this->items[$offset]); }
}

$refBag = new MagicReferenceBag();
$plainStorage = ["slot" => "plain", "nested" => ["leaf" => "inside"]];

class MagicReferenceArrayAccessBox {
    public function &__get($name) {
        global $refBag;
        return $refBag;
    }
}

class MagicPlainArrayBox {
    public function &__get($name) {
        global $plainStorage;
        return $plainStorage;
    }
}

$box = new MagicReferenceArrayAccessBox();
$alias =& $box->missing["slot"];
$alias = "changed";
echo "ref-offset:", $alias, "|", $refBag->items["slot"], "\n";

$append =& $box->missing[];
$append = "append-changed";
echo "ref-append:", $append, "|", $refBag->items[""], "\n";

$plainBox = new MagicPlainArrayBox();
$plain =& $plainBox->missing["slot"];
$plain = "plain-changed";
echo "plain-offset:", $plain, "|", $plainStorage["slot"], "\n";

$nested =& $plainBox->missing["nested"]["leaf"];
$nested = "nested-changed";
echo "plain-nested:", $nested, "|", $plainStorage["nested"]["leaf"], "\n";

$plainAppend =& $plainBox->missing[];
$plainAppend = "plain-append";
echo "plain-append:", $plainAppend, "|", $plainStorage[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "ref-offset:changed|changed\n",
            "ref-append:append-changed|append-changed\n",
            "plain-offset:plain-changed|plain-changed\n",
            "plain-nested:nested-changed|nested-changed\n",
            "plain-append:plain-append|plain-append"
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_parameter_accepts_non_direct_holder_magic_get_array_offset_source() {
    let execution = run_source(
        r#"<?php
$storage = ["slot" => "initial", "nested" => ["leaf" => "inside"], "dynamic" => "selected"];

class NonDirectMagicArrayBox {
    public function &__get($name) {
        echo "get:$name\n";
        global $storage;
        return $storage;
    }
}

function touch_non_direct_magic_slot(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

$holders = ["box" => new NonDirectMagicArrayBox()];
touch_non_direct_magic_slot($holders["box"]->missing["slot"], "param");
echo $storage["slot"], "\n";

$alias =& $holders["box"]->missing["nested"]["leaf"];
$alias = $alias . ":alias";
echo $storage["nested"]["leaf"], "|";
$storage["nested"]["leaf"] = $storage["nested"]["leaf"] . ":store";
echo $alias, "\n";

$property = "dynamicMissing";
touch_non_direct_magic_slot($holders["box"]->{$property}["dynamic"], "dynamic");
echo $storage["dynamic"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "get:missing\n",
            "initial:param\n",
            "get:missing\n",
            "inside:alias|inside:alias:store\n",
            "get:dynamicMissing\n",
            "selected:dynamic"
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_accepts_non_direct_holder_magic_get_array_append_source() {
    let execution = run_source(
        r#"<?php
$storage = ["nested" => ["base" => "keep"]];

class NonDirectMagicAppendBox {
    public function &__get($name) {
        echo "get:$name\n";
        global $storage;
        return $storage;
    }
}

$holders = ["box" => new NonDirectMagicAppendBox()];
$alias =& $holders["box"]->missing[];
$alias = "first";
echo $storage[0], "|", $alias, "\n";
$storage[0] = "store";
echo $alias, "|";
$alias = "tail";
echo $storage[0], "\n";
unset($alias);

$nested =& $holders["box"]->missing["nested"][];
$nested = "child";
echo $storage["nested"][0], "|", $nested, "\n";

$property = "dynamicMissing";
$dynamic =& $holders["box"]->{$property}[];
$dynamic = "dynamic";
echo $storage[1], "|", $dynamic;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "get:missing\n",
            "first|first\n",
            "store|tail\n",
            "get:missing\n",
            "child|child\n",
            "get:dynamicMissing\n",
            "dynamic|dynamic"
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_parameter_accepts_inaccessible_declared_property_magic_get_source() {
    let execution = run_source(
        r#"<?php
$storage = "initial";
$dynamicStorage = "dynamic";
$items = ["slot" => "array"];

class InaccessibleMagicBox {
    private $secret = "hidden";

    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

class InaccessibleDynamicMagicBox {
    protected $secret = "hidden";

    public function &__get($name) {
        global $dynamicStorage;
        return $dynamicStorage;
    }
}

class InaccessibleMagicArrayBox {
    private $items = [];

    public function &__get($name) {
        global $items;
        return $items;
    }
}

function mutate_inaccessible_magic(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
}

$box = new InaccessibleMagicBox();
mutate_inaccessible_magic($box->secret, "private");

$property = "secret";
$magicBox = new InaccessibleDynamicMagicBox();
mutate_inaccessible_magic($magicBox->{$property}, "dynamic");

$arrayBox = new InaccessibleMagicArrayBox();
mutate_inaccessible_magic($arrayBox->items["slot"], "array");

echo $storage, "\n", $dynamicStorage, "\n", $items["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "initial:private\ndynamic:dynamic\narray:array"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn normal_reference_return_call_accepts_magic_get_array_offset_argument() {
    let execution = run_source(
        r#"<?php
$storage = ["slot" => "initial", "nested" => ["leaf" => "inside"]];

class MagicReturnCallBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

class MagicReturnCallPicker {
    public function &touch(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function &touch_magic_return_call(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$box = new MagicReturnCallBox();
touch_magic_return_call($box->missing["slot"], "function");

$picker = new MagicReturnCallPicker();
$picker->touch($box->missing["nested"]["leaf"], "method");

echo $storage["slot"], "\n", $storage["nested"]["leaf"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "initial:function\ninside:method");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn normal_static_reference_return_call_reads_returned_cell_by_value() {
    let execution = run_source(
        r#"<?php
class StaticReferenceTouch {
    public static function &touch(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class StaticReferenceParent {
    public static function &touch(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class StaticReferenceChild extends StaticReferenceParent {
    public function run(&$value) {
        self::touch($value, "self");
        parent::touch($value, "parent");
        static::touch($value, "static");
    }

    public static function &touch(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

$magicStorage = ["slot" => "magic"];

class StaticReferenceMagicBox {
    public function &__get($name) {
        global $magicStorage;
        return $magicStorage;
    }
}

$items = [
    "named" => "named",
    "class_string" => "class",
    "object" => "object",
    "context" => "context",
];

StaticReferenceTouch::touch($items["named"], "direct");

$class = "StaticReferenceTouch";
$class::touch($items["class_string"], "dynamic");

$magicBox = new StaticReferenceMagicBox();
StaticReferenceTouch::touch($magicBox->missing["slot"], "magic");
$class::touch($magicBox->missing["slot"], "dynamic_magic");

$object = new StaticReferenceTouch();
$object::touch($items["object"], "object");

$child = new StaticReferenceChild();
$child->run($items["context"]);

echo $items["named"], "\n";
echo $items["class_string"], "\n";
echo $items["object"], "\n";
echo $items["context"], "\n";
echo $magicStorage["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "named:direct\nclass:dynamic\nobject:object\ncontext:self:parent:static\nmagic:magic:dynamic_magic"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_source_inside_unexecuted_body_is_registered() {
    let execution = run_source(
        r#"<?php
class Query {
    public $posts;

    public function register() {
        $GLOBALS["posts"] =& $this->posts;
    }
}
echo "loaded";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "loaded");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_source_copies_current_container_value() {
    let execution = run_source(
        r#"<?php
class Query {
    public $posts;
}

$query = new Query();
$query->posts = ["first"];
$GLOBALS["posts"] =& $query->posts;
echo $GLOBALS["posts"][0];
$query->posts[0] = "changed";
echo "|", $GLOBALS["posts"][0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|first");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_public_object_property_source_aliases_direct_variable() {
    let execution = run_source(
        r#"<?php
class Box {
    public $value = "initial";
}

$box = new Box();
$alias =& $box->value;
$alias = "from-alias";
echo $box->value;
echo "|";
$box->value = "from-property";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-property");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_public_object_property_array_source_aliases_direct_variable() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = ["first"];
}

$box = new Box();
$alias =& $box->items;
$alias[0] = "from-alias";
echo $box->items[0];
echo "|";
$box->items = ["from-property"];
echo $alias[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-property");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_dynamic_public_object_property_source_aliases_direct_variable() {
    let execution = run_source(
        r#"<?php
class Box {
    public $value = "initial";
}

$box = new Box();
$property = "value";
$alias =& $box->$property;
$alias = "from-alias";
echo $box->value;
echo "|";
$box->$property = "from-property";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-property");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_dynamic_stdclass_property_source_materializes_missing_slot() {
    let execution = run_source(
        r#"<?php
$box = new stdClass();
$property = "created";
$alias =& $box->$property;
echo $alias === null ? "null" : "not-null";
echo "|";
$alias = "from-alias";
echo $box->created;
echo "|";
$box->$property = "from-property";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "null|from-alias|from-property");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_dynamic_declared_object_missing_property_source_remains_boundary() {
    let error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$property = "missing";
$alias =& $box->$property;
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined property Box::$missing");
}

#[test]
fn reference_assignment_magic_get_source_aliases_direct_variable_return() {
    let execution = run_source(
        r#"<?php
$storage = "initial";

class MagicBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

$box = new MagicBox();
$alias =& $box->missing;
$alias = "from-alias";
echo $storage;
echo "|";
$storage = "from-global";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-global");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_dynamic_magic_get_source_aliases_direct_variable_return() {
    let execution = run_source(
        r#"<?php
$storage = "initial";

class MagicBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

$box = new MagicBox();
$property = "missing";
$alias =& $box->$property;
$alias = "from-alias";
echo $storage;
echo "|";
$storage = "from-global";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-global");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_inaccessible_declared_property_magic_get_source_aliases_cell() {
    let execution = run_source(
        r#"<?php
$storage = "initial";

class InaccessibleMagicAliasBox {
    private $secret = "hidden";

    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

$box = new InaccessibleMagicAliasBox();
$alias =& $box->secret;
$alias = "from-alias";
echo $storage;
echo "|";
$storage = "from-global";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-global");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_magic_get_source_without_reference_return_remains_boundary() {
    let error = runtime_error(
        r#"<?php
class MagicBox {
    public function __get($name) {
        return "value";
    }
}

$box = new MagicBox();
$alias =& $box->missing;
"#,
    );

    assert_eq!(error.line, 9);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call MagicBox::__get(): magic __get reference sources require __get() to return by reference in the current subset"
    );
}

#[test]
fn reference_assignment_magic_get_source_non_direct_return_remains_boundary() {
    let error = runtime_error(
        r#"<?php
class MagicBox {
    public $value = "initial";

    public function &__get($name) {
        return $this->value;
    }
}

$box = new MagicBox();
$alias =& $box->missing;
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 9);
    assert_eq!(
        error.message,
        "unsupported call __get(): reference returns are only implemented for direct variable return expressions"
    );
}

#[test]
fn reference_assignment_non_public_object_property_source_aliases_inside_method_context() {
    let execution = run_source(
        r#"<?php
class Box {
    private $secret = "initial";
    protected $label = "start";

    public function run() {
        $secret =& $this->secret;
        $secret = "secret-alias";
        echo $this->secret;
        echo "|";
        $this->secret = "secret-property";
        echo $secret;
        echo "|";

        $label =& $this->label;
        $label = "label-alias";
        echo $this->label;
        echo "|";
        $this->label = "label-property";
        echo $label;
    }
}

$box = new Box();
$box->run();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "secret-alias|secret-property|label-alias|label-property"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_private_property_source_outside_context_remains_boundary() {
    let error = runtime_error(
        r#"<?php
class Box {
    private $secret = "initial";
}

$box = new Box();
$alias =& $box->secret;
"#,
    );

    assert_eq!(error.line, 7);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$secret requires same-class method context in the current subset"
    );
}

#[test]
fn reference_assignment_inherited_protected_property_source_aliases_inside_child_context() {
    let execution = run_source(
        r#"<?php
class Base {
    protected $shared = "base";

    public function readShared() {
        return $this->shared;
    }
}

class Child extends Base {
    public function aliasOwn() {
        $alias =& $this->shared;
        $alias = "own-alias";
        echo $this->shared;
        echo "|";
        $this->shared = "own-property";
        echo $alias;
    }

    public function aliasPeer($other) {
        $alias =& $other->shared;
        $alias = "peer-alias";
        echo $other->readShared();
        echo "|";
        $other->shared = "peer-property";
        echo $alias;
    }
}

$child = new Child();
$child->aliasOwn();
echo "|";
$peer = new Child();
$child->aliasPeer($peer);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "own-alias|own-property|peer-alias|peer-property"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_public_object_property_array_offset_source_aliases_inside_method_context(
) {
    let execution = run_source(
        r#"<?php
class Box {
    private $privateItems = ["slot" => "private"];
    protected $protectedItems = ["slot" => "protected"];

    public function run($key) {
        $private =& $this->privateItems[$key];
        $private = "private-alias";
        echo $this->privateItems[$key];
        echo "|";
        $this->privateItems[$key] = "private-property";
        echo $private;
        echo "|";

        $protected =& $this->protectedItems[$key];
        $protected = "protected-alias";
        echo $this->protectedItems[$key];
        echo "|";
        $this->protectedItems[$key] = "protected-property";
        echo $protected;
    }
}

$box = new Box();
$box->run("slot");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "private-alias|private-property|protected-alias|protected-property"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_protected_peer_object_property_array_offset_source_aliases_inside_child_context(
) {
    let execution = run_source(
        r#"<?php
class Base {
    protected $items = ["slot" => "base"];

    public function readItem($key) {
        return $this->items[$key];
    }
}

class Child extends Base {
    public function aliasPeer($other, $key) {
        $alias =& $other->items[$key];
        $alias = "peer-alias";
        echo $other->readItem($key);
        echo "|";
        $other->items[$key] = "peer-property";
        echo $alias;
    }
}

$child = new Child();
$peer = new Child();
$child->aliasPeer($peer, "slot");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "peer-alias|peer-property");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_public_object_property_array_offset_source_materializes_null_property()
{
    let execution = run_source(
        r#"<?php
class Box {
    private $privateItems;
    protected $protectedItems = [];

    public function run($key) {
        $private =& $this->privateItems[$key];
        $private = "private-created";
        echo $this->privateItems[$key];
        echo "|";
        $protected =& $this->protectedItems[$key];
        $protected = "protected-created";
        echo $this->protectedItems[$key];
    }
}

$box = new Box();
$box->run("slot");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "private-created|protected-created");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_public_object_property_array_offset_source_outside_context_remains_boundary(
) {
    let error = runtime_error(
        r#"<?php
class Box {
    private $items = ["slot" => "private"];
}

$box = new Box();
$alias =& $box->items["slot"];
"#,
    );

    assert_eq!(error.line, 7);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$items requires same-class method context in the current subset"
    );
}

#[test]
fn reference_assignment_non_public_object_property_array_append_source_aliases_inside_method_context(
) {
    let execution = run_source(
        r#"<?php
class Box {
    private $privateItems = [];
    protected $protectedItems = [];

    public function run() {
        $private =& $this->privateItems[];
        $private = "private-alias";
        echo $this->privateItems[0];
        echo "|";
        $this->privateItems[0] = "private-property";
        echo $private;
        echo "|";

        $protected =& $this->protectedItems[];
        $protected = "protected-alias";
        echo $this->protectedItems[0];
        echo "|";
        $this->protectedItems[0] = "protected-property";
        echo $protected;
    }
}

$box = new Box();
$box->run();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "private-alias|private-property|protected-alias|protected-property"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_protected_peer_object_property_array_append_source_aliases_inside_child_context(
) {
    let execution = run_source(
        r#"<?php
class Base {
    protected $items = [];

    public function readItem($key) {
        return $this->items[$key];
    }
}

class Child extends Base {
    public function aliasPeer($other) {
        $alias =& $other->items[];
        $alias = "peer-alias";
        echo $other->readItem(0);
        echo "|";
        $other->items[0] = "peer-property";
        echo $alias;
    }
}

$child = new Child();
$peer = new Child();
$child->aliasPeer($peer);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "peer-alias|peer-property");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_public_object_property_array_append_source_materializes_null_and_parent_path(
) {
    let execution = run_source(
        r#"<?php
class Box {
    private $privateItems;
    protected $protectedItems = [];

    public function run($outer) {
        $private =& $this->privateItems[];
        $private = "private-created";
        echo $this->privateItems[0];
        echo "|";

        $protected =& $this->protectedItems[$outer][];
        $protected = "protected-created";
        echo $this->protectedItems[$outer][0];
    }
}

$box = new Box();
$box->run("outer");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "private-created|protected-created");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_non_public_object_property_array_append_source_outside_context_remains_boundary(
) {
    let error = runtime_error(
        r#"<?php
class Box {
    private $items = [];
}

$box = new Box();
$alias =& $box->items[];
"#,
    );

    assert_eq!(error.line, 7);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$items requires same-class method context in the current subset"
    );
}

#[test]
fn reference_assignment_object_property_array_offset_source_aliases_direct_slot() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = ["slot" => "x"];
}
$box = new Box();
$alias =& $box->items["slot"];
$alias = "from-alias";
echo $box->items["slot"];
echo "|";
$box->items["slot"] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_offset_source_aliases_direct_slot_with_key() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = ["slot" => "x"];
}
$box = new Box();
$key = "slot";
$alias =& $box->items[$key];
$alias = "from-alias";
echo $box->items["slot"];
echo "|";
$box->items["slot"] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_offset_source_materializes_missing_and_null_slots() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items;
}
$box = new Box();
$missing =& $box->items["missing"];
$missing = "created";
echo $box->items["missing"];
echo "|";
$box->items["missing"] = "updated";
echo $missing;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "created|updated");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_offset_source_non_array_boundary_is_stable() {
    let error = runtime_error(
        r#"<?php
class Box {
    public $items = "string";
}
$box = new Box();
$alias =& $box->items["slot"];
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot read offset on string"
    );
}

#[test]
fn reference_assignment_object_property_nested_array_offset_source_aliases_direct_slot() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = ["outer" => ["inner" => "x"]];
}
$box = new Box();
$alias =& $box->items["outer"]["inner"];
$alias = "from-alias";
echo $box->items["outer"]["inner"];
echo "|";
$box->items["outer"]["inner"] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_nested_array_offset_source_materializes_path() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items;
}
$box = new Box();
$alias =& $box->items["outer"]["inner"];
$alias = "created";
echo $box->items["outer"]["inner"];
echo "|";
$box->items["outer"]["inner"] = "updated";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "created|updated");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_array_append_source_aliases_appended_slot() {
    let execution = run_source(
        r#"<?php
$items = [];
$alias =& $items[];
$alias = "from-alias";
echo $items[0];
echo "|";
$items[0] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_nested_array_append_source_aliases_appended_slot() {
    let execution = run_source(
        r#"<?php
$items = [];
$alias =& $items["outer"][];
$alias = "from-alias";
echo $items["outer"][0];
echo "|";
$items["outer"][0] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_array_append_source_aliases_appended_slot() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = [];
}
$box = new Box();
$alias =& $box->items[];
$alias = "from-alias";
echo $box->items[0];
echo "|";
$box->items[0] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_object_property_nested_array_append_source_aliases_appended_slot() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items;
}
$box = new Box();
$alias =& $box->items["outer"][];
$alias = "from-alias";
echo $box->items["outer"][0];
echo "|";
$box->items["outer"][0] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_rebinds_alias_backed_array_slots_to_new_sources() {
    let execution = run_source(
        r#"<?php
$_REQUEST["payload"] = array("slot" => "request-old", "append" => array());
$payload =& $_REQUEST["payload"];
$value = "request-new";
$payload["slot"] =& $value;
$value = "request-source";
echo $_REQUEST["payload"]["slot"], "|";
$_REQUEST["payload"]["slot"] = "request-target";
echo $value, "\n";

class ReferenceAssignmentBox {
    public $items = array("slot" => "box-old", "nested" => array("slot" => "nested-old"));
}

$box = new ReferenceAssignmentBox();
$slot =& $box->items["slot"];
$boxValue = "box-new";
$slot =& $boxValue;
$boxValue = "box-source";
echo $box->items["slot"], "|";
$box->items["slot"] = "box-target";
echo $boxValue, "\n";

$nested =& $box->items["nested"];
$nestedValue = "nested-new";
$nested["slot"] =& $nestedValue;
$nestedValue = "nested-source";
echo $box->items["nested"]["slot"], "|";
$box->items["nested"]["slot"] = "nested-target";
echo $nestedValue;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request-source|request-target\nbox-old|box-source\nnested-source|nested-target"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reference_assignment_complex_object_property_array_source_boundary_is_stable() {
    let error = runtime_error(
        r#"<?php
$alias =& make_box()->items[0];
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 11);
    assert_eq!(error.message, "undefined function make_box()");
}

#[test]
fn standalone_reference_expressions_still_have_stable_parse_error() {
    let error = parse_error(
        r#"<?php
$value = 1;
echo &$value;
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported reference expression: references are not implemented"
    );
}

#[test]
fn closure_syntax_inside_unexecuted_function_body_is_registered() {
    let execution = run_source(
        r#"<?php
function register_handler() {
    $utf8_pcre = null;
    $handler = function ($errno, $errstr) use (&$utf8_pcre) {
        $utf8_pcre = false;
        return false;
    };
    return "registered";
}
echo "ok";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ok");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn anonymous_closure_values_can_be_assigned_without_invocation() {
    let execution = run_source(
        r#"<?php
$fn = function ($value) {
    return $value;
};
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn static_anonymous_closure_values_can_be_assigned_without_invocation() {
    let execution = run_source(
        r#"<?php
$fn = static function ($value) {
    echo "body";
    return $value;
};
if ($fn) {
    echo "truthy";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "truthy");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn anonymous_closure_capture_binding_allocates_current_inert_closure_values() {
    let execution = run_source(
        r#"<?php
$value = 1;
$ref = 2;
$fn = function () use ($value) {
    return $value;
};
$byRef = function () use (&$ref) {
    return $ref;
};
echo $fn ? "value" : "missing";
echo "|";
echo $byRef ? "ref" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "value|ref");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn static_anonymous_closure_capture_binding_allocates_current_inert_closure_values() {
    let execution = run_source(
        r#"<?php
$value = 1;
$fn = static function () use ($value) {
    return $value;
};
echo $fn ? "truthy" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "truthy");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn anonymous_closure_invocation_executes_current_body_subset() {
    let execution = run_source(
        r#"<?php
$fn = function () {
    return 1;
};
echo $fn();
"#,
    );

    let execution = execution.unwrap();
    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn static_anonymous_closure_invocation_executes_current_body_subset() {
    let execution = run_source(
        r#"<?php
$fn = static function () {
    return 1;
};
echo $fn();
"#,
    );

    let execution = execution.unwrap();
    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn static_anonymous_closure_is_not_callable_in_current_runtime_subset() {
    let execution = run_source(
        r#"<?php
$fn = static function () {
    return 1;
};
echo is_callable($fn) ? "callable" : "not-callable";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "not-callable");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn arrow_function_values_can_be_assigned_without_invocation() {
    let execution = run_source(
        r#"<?php
$fn = fn($value) => $value;
if ($fn) {
    echo "truthy\n";
}
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "truthy\nafter");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn arrow_function_invocation_has_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
$fn = fn($value) => $value;
echo $fn("Ada");
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call closure: arrow closure invocation is not implemented"
    );
}

#[test]
fn variadic_argument_unpacking_is_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
function first($value) {
    return $value;
}
$items = [1];
echo first(...$items);
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported argument unpacking: call-site ... expansion requires iterable unpacking order, string-keyed named-argument interaction, by-reference argument propagation, variadic collection, duplicate argument diagnostics, and native lowering"
    );
}

#[test]
fn named_arguments_are_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
function greet($name) {
    return $name;
}
echo greet(name: "Ada");
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported named argument: call argument names require parameter-name metadata, duplicate and unknown-name diagnostics, positional/named ordering, by-reference binding, variadic collection, unpacking interaction, and native lowering"
    );
}

#[test]
fn strict_types_declare_is_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
declare(strict_types=1);
function identity($value) {
    return $value;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported declare directive: strict_types is not implemented"
    );
}

#[test]
fn unsupported_declare_directives_have_directive_specific_parse_errors() {
    let cases = [
        (
            r#"<?php
declare(ticks=1);
echo "unreachable";
"#,
            "unsupported declare directive: ticks requires tick handlers and execution hooks, which are not implemented",
        ),
        (
            r#"<?php
declare(encoding="UTF-8");
echo "unreachable";
"#,
            "unsupported declare directive: encoding requires source encoding, lexer decoding, and runtime text handling, which are not implemented",
        ),
    ];

    for (source, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 1);
        assert_eq!(error.message, message);
    }
}
