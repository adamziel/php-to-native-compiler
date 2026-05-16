use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::{run_source, run_source_with_source_file};

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
        "unsupported call mutate(): reference parameter invocation is only implemented for direct variable arguments in the current subset"
    );
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
        "unsupported magic constant __TRAIT__: trait context evaluation requires trait declarations, trait use, and trait-context tracking, which are not implemented"
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
fn reference_return_invocation_reports_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
function &identity($value) {
    return $value;
}
echo identity(1);
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call identity(): reference returns are not implemented"
    );
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
fn reference_return_method_invocation_reports_stable_runtime_boundary() {
    let error = runtime_error(
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
    );

    assert_eq!(error.line, 9);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call make(): reference returns are not implemented"
    );
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
fn reference_assignment_array_offset_target_aliased_source_remains_boundary() {
    let error = runtime_error(
        r#"<?php
$value = "source";
$other =& $value;
$items["slot"] =& $value;
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: array-offset reference targets cannot rebind an existing direct variable alias group"
    );
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
fn reference_assignment_array_append_target_aliased_source_remains_boundary() {
    let error = runtime_error(
        r#"<?php
$value = "source";
$other =& $value;
$items[] =& $value;
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: array-offset reference targets cannot rebind an existing direct variable alias group"
    );
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
fn reference_assignment_nested_array_offset_target_aliased_source_remains_boundary() {
    let error = runtime_error(
        r#"<?php
$value = "source";
$other =& $value;
$items["outer"]["slot"] =& $value;
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: array-offset reference targets cannot rebind an existing direct variable alias group"
    );
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
fn reference_assignment_object_property_array_target_executes_as_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
class Catalog {
    public $entries;

    public function run() {
        $entry = 1;
        $this->entries[$entry] =& $entry;
    }
}
$catalog = new Catalog();
$catalog->run();
"#,
    );

    assert_eq!(error.line, 7);
    assert_eq!(error.column, 9);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: references and aliasing are not implemented"
    );
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
fn reference_assignment_source_boundary_is_stable() {
    let error = parse_error(
        r#"<?php
$alias =& $box->items[0];
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 11);
    assert_eq!(
        error.message,
        "unsupported reference assignment: only direct variable, direct array-offset, object-property, function-call, and method-call reference sources are parsed before reference semantics exist"
    );
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
fn anonymous_closure_invocation_has_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
$fn = function () {
    return 1;
};
echo $fn();
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call closure: closure invocation is not implemented"
    );
}

#[test]
fn static_anonymous_closure_invocation_has_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
$fn = static function () {
    return 1;
};
echo $fn();
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call closure: closure invocation is not implemented"
    );
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
        "unsupported call closure: closure invocation is not implemented"
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
        "unsupported argument unpacking: variadic calls are not implemented"
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
        "unsupported named argument: named arguments are not implemented"
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
