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
fn reference_parameter_invocation_is_rejected_until_references_exist() {
    let error = runtime_error(
        r#"<?php
function mutate(&$value) {
    $value = 2;
}
$value = 1;
mutate($value);
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mutate(): reference parameter invocation is not implemented"
    );
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
fn magic_class_constant_is_rejected_until_class_context_tracking_exists() {
    let error = parse_error(
        r#"<?php
class Box {
    public function label() {
        return __CLASS__;
    }
}
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 16);
    assert_eq!(
        error.message,
        "unsupported magic constant __CLASS__: class context evaluation requires class-context tracking, which is not implemented"
    );
}

#[test]
fn magic_method_constant_is_rejected_until_method_dispatch_exists() {
    let error = parse_error(
        r#"<?php
class Box {
    public function label() {
        return __METHOD__;
    }
}
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 16);
    assert_eq!(
        error.message,
        "unsupported magic constant __METHOD__: method context evaluation requires method dispatch, which is not implemented"
    );
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
fn reference_assignment_executes_as_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
$value = 1;
$alias =& $value;
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: references and aliasing are not implemented"
    );
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
fn reference_assignment_array_offset_source_executes_as_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
$items = [1];
$alias =& $items[0];
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: references and aliasing are not implemented"
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
    assert_eq!(error.column, 9);
    assert_eq!(
        error.message,
        "unsupported call reference assignment: references and aliasing are not implemented"
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
fn reference_assignment_source_boundary_is_stable() {
    let error = parse_error(
        r#"<?php
$alias =& make_value();
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 11);
    assert_eq!(
        error.message,
        "unsupported reference assignment: only direct variable, direct array-offset, and method-call reference sources are parsed before reference semantics exist"
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
fn anonymous_closure_capture_binding_has_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
$value = 1;
$fn = function () use ($value) {
    return $value;
};
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "unsupported call closure: closure capture binding is not implemented"
    );
}

#[test]
fn static_anonymous_closure_capture_binding_has_stable_runtime_boundary() {
    let error = runtime_error(
        r#"<?php
$value = 1;
$fn = static function () use ($value) {
    return $value;
};
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "unsupported call closure: closure capture binding is not implemented"
    );
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
