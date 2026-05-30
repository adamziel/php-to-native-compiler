use std::fs;

use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::{run_source, run_source_with_source_file};

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn lex_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Lex);
    error
}

#[test]
fn materialized_symbol_table_preserves_static_variable_behavior() {
    let execution = run_source(
        r#"<?php
$name = "Ada";
$label = $name . "-static";
$items = [];
$items["label"] = $label;
echo isset($name), "\n";
echo $items["label"], "\n";
function shadow($name = "local") {
    $name = $name . "-scope";
    return $name;
}
echo shadow(), "\n";
echo $name, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\nAda-static\nlocal-scope\nAda\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_function_calls_use_runtime_lookup_for_string_callees() {
    let execution = run_source(
        r#"<?php
function greet($name, $suffix = "!") {
    return "hello " . $name . $suffix;
}
$call = "greet";
echo $call("Ada"), "\n";
$upper = "GREET";
echo $upper("Lin", "."), "\n";
$length = "strlen";
echo $length("native"), "\n";
$counter = "count";
echo $counter(["a", "b"]), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello Ada!\nhello Lin.\n6\n2\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_function_calls_use_exact_namespaced_string_callees() {
    let execution = run_source(
        r#"<?php
namespace App;
function greet($name) {
    return "hello " . $name;
}
$call = "App\\greet";
echo $call("Ada"), "\n";

$local = "greet";
echo function_exists($local) ? "yes" : "no";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello Ada\nno");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_function_calls_accept_optional_trailing_commas() {
    let execution = run_source(
        r#"<?php
function greet($name, $suffix = "!") {
    return "hello " . $name . $suffix;
}
$call = "greet";
echo $call("Ada",), "\n";
echo $call("Lin", ".",), "\n";
$length = "strlen";
echo $length("native",), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello Ada!\nhello Lin.\n6\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_callable_array_and_closure_spread_arguments_are_expanded() {
    let execution = run_source(
        r#"<?php
class DynamicSpreadTarget {
    public static function withArgs($left, $middle, $right) {
        echo $left, "|", $middle, "|", $right, "\n";
    }
}

$tail = ["B", "C"];
$callback = ["DynamicSpreadTarget", "withArgs"];
$callback("A", ...$tail);

$closure = function($first, $second, $third) {
    echo $first, ":", $second, ":", $third, "\n";
};
$closure(...[1, 2, 3]);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "A|B|C\n1:2:3\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn closure_argument_unpacking_accepts_string_keys_as_named_arguments() {
    let execution = run_source(
        r#"<?php
$join = function($left, $middle = "M", $right = "R") {
    echo $left, ":", $middle, ":", $right;
};

$args = ["right" => "C", "middle" => "B"];
$join("A", ...$args);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "A:B:C");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_argument_unpacking_reuses_source_order_named_binding() {
    let execution = run_source(
        r#"<?php
function describe_call_unpack($first, $second = "D", $third = "T") {
    echo $first, "|", $second, "|", $third, "\n";
}

$args = ["third" => "C", "first" => "A"];
call_user_func("describe_call_unpack", ...$args, second: "B");

function collect_call_unpack(...$args) {
    echo count($args), ":", $args[0], ":", $args["a"], ":", $args["b"];
}

call_user_func("collect_call_unpack", ...[1, "a" => 2], b: 3);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "A|B|C\n3:1:2:3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_func_get_builtins_read_active_user_call_frame() {
    let execution = run_source(
        r#"<?php
function fixed_frame($a = "A", $b = "B") {
    $a = "changed";
    echo func_num_args(), "|", implode(",", func_get_args()), "|", func_get_arg(0), "|", func_get_arg(1), "\n";
}

function variadic_frame($a, ...$rest) {
    $a = "first";
    $rest[0] = "mutated-rest";
    echo func_num_args(), "|", implode(",", func_get_args()), "\n";
}

fixed_frame(b: "Bee");
variadic_frame("A", "B", "C");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "2|changed,Bee|changed|Bee\n3|first,B,C\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn func_get_builtins_cover_extra_args_global_errors_and_fq_constants() {
    let execution = run_source(
        r#"<?php
function extra_frame($a) {
    var_dump(func_num_args());
    var_dump(func_get_args());
    var_dump(func_get_arg(1));
    try {
        func_get_arg(3);
    } catch (\Error $e) {
        echo $e->getMessage(), \PHP_EOL;
    }
    try {
        func_get_arg(-1);
    } catch (\ValueError $e) {
        echo $e->getMessage(), \PHP_EOL;
    }
}

extra_frame(1, 2, 3);

try {
    func_get_args();
} catch (\Error $e) {
    echo $e->getMessage(), \PHP_EOL;
}

try {
    func_num_args();
} catch (\Error $e) {
    echo $e->getMessage(), \PHP_EOL;
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(3)\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  int(1)\n",
            "  [1]=>\n",
            "  int(2)\n",
            "  [2]=>\n",
            "  int(3)\n",
            "}\n",
            "int(2)\n",
            "func_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function\n",
            "func_get_arg(): Argument #1 ($position) must be greater than or equal to 0\n",
            "func_get_args() cannot be called from the global scope\n",
            "func_num_args() must be called from a function context\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn generator_rewind_foreach_and_func_get_use_materialized_yields() {
    let execution = run_source(
        r#"<?php
function sample_generator($first, $second = "B") {
    echo "materialize\n";
    $first = "changed";
    yield func_num_args();
    yield "args" => implode(",", func_get_args());
    yield "alpha";
    yield 5 => "five";
    yield "name" => "named";
}

echo "before\n";
$gen = sample_generator("A", "B", "C");
echo "after-call\n";
$gen->rewind();
var_dump($gen->key(), $gen->current(), $gen->valid());
$gen->next();
var_dump($gen->key(), $gen->current());

foreach (sample_generator("X") as $key => $value) {
    echo $key, ":", $value, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "before\n",
            "after-call\n",
            "materialize\n",
            "int(0)\n",
            "int(3)\n",
            "bool(true)\n",
            "string(4) \"args\"\n",
            "string(11) \"changed,B,C\"\n",
            "materialize\n",
            "0:1\n",
            "args:changed\n",
            "1:alpha\n",
            "5:five\n",
            "name:named\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn generator_yield_from_arrays_and_generators_materializes_values() {
    let execution = run_source(
        r#"<?php
function inner_values() {
    yield "start";
    yield from [];
    yield from ["left", "right"];
}

function outer_values() {
    yield "outer";
    yield from inner_values();
    yield from ["done"];
    yield "tail";
}

foreach (outer_values() as $key => $value) {
    echo $key, ":", $value, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "0:outer\n",
            "0:start\n",
            "0:left\n",
            "1:right\n",
            "0:done\n",
            "1:tail\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn generator_get_return_reads_materialized_return_value() {
    let execution = run_source(
        r#"<?php
function immediate_return() {
    return 42;
    yield 24;
}

function return_after_yield($value) {
    yield "first";
    return $value;
}

function implicit_null_return() {
    yield "once";
}

function typed_value_less_return() : Generator {
    return;
    yield "never";
}

function &by_reference_generator_declaration() {
    $value = "ref-yield-syntax";
    yield $value;
    return "ref-return";
}

$immediate = immediate_return();
var_dump($immediate->getReturn());

$after = return_after_yield("done");
var_dump($after->current());
$after->next();
var_dump($after->getReturn());

$implicit = implicit_null_return();
var_dump($implicit->current());
$implicit->next();
var_dump($implicit->getReturn());

$typed = typed_value_less_return();
var_dump($typed->getReturn());

$reflection = new ReflectionFunction("by_reference_generator_declaration");
var_dump($reflection->returnsReference());
$byRefDeclaration = by_reference_generator_declaration();
var_dump($byRefDeclaration->current());
$byRefDeclaration->next();
var_dump($byRefDeclaration->getReturn());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(42)\n",
            "string(5) \"first\"\n",
            "string(4) \"done\"\n",
            "string(4) \"once\"\n",
            "NULL\n",
            "NULL\n",
            "bool(true)\n",
            "string(16) \"ref-yield-syntax\"\n",
            "string(10) \"ref-return\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn generator_throw_aborts_generator_and_remains_catchable() {
    let source = r#"<?php
function throwing_before_yield() {
    throw new Exception("before");
    yield 1;
    return 2;
}

$gen = throwing_before_yield();
try {
    $gen->next();
} catch (Exception $e) {
    echo $e->getMessage(), "\n";
}
try {
    $gen->getReturn();
} catch (Exception $e) {
    echo $e->getMessage(), "\n";
}

function injected_throw() {
    yield;
    return "done";
}

$gen = injected_throw();
try {
    $gen->throw(new Exception("injected"));
} catch (Exception $e) {
    echo $e->getMessage(), "\n";
}
try {
    $gen->getReturn();
} catch (Exception $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "before\nCannot get return value of a generator that hasn't returned\ninjected\nCannot get return value of a generator that hasn't returned\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn generator_yield_keys_preserve_nonscalar_values() {
    let execution = run_source(
        r#"<?php
function nonscalar_keys() {
    yield [] => 1;
    yield [1, 2] => [3, 4];
    yield 3.14 => "pi";
    yield false => "false-key";
    yield true => "true-key";
    yield null => "null-key";
}

foreach (nonscalar_keys() as $key => $value) {
    var_dump($key, $value);
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array(0) {\n",
            "}\n",
            "int(1)\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  int(1)\n",
            "  [1]=>\n",
            "  int(2)\n",
            "}\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  int(3)\n",
            "  [1]=>\n",
            "  int(4)\n",
            "}\n",
            "float(3.14)\n",
            "string(2) \"pi\"\n",
            "bool(false)\n",
            "string(9) \"false-key\"\n",
            "bool(true)\n",
            "string(8) \"true-key\"\n",
            "NULL\n",
            "string(8) \"null-key\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn forbidden_scope_introspection_builtins_report_dynamic_call_error() {
    let execution = run_source(
        r#"<?php
try {
    $call = "extract";
    $call(["a" => 1]);
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $call = "compact";
    $call("a");
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $call = "get_defined_vars";
    $call();
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $call = "func_get_args";
    $call();
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $call = "func_get_arg";
    $call(0);
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    $call = "func_num_args";
    $call();
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Cannot call extract() dynamically\n",
            "Cannot call compact() dynamically\n",
            "Cannot call get_defined_vars() dynamically\n",
            "Cannot call func_get_args() dynamically\n",
            "Cannot call func_get_arg() dynamically\n",
            "Cannot call func_num_args() dynamically\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn forbidden_dynamic_builtins_are_rejected_through_callback_dispatchers() {
    let execution = run_source(
        r#"<?php
try {
    array_map("extract", [["i" => 1]]);
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    call_user_func("extract", ["i" => 1]);
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
try {
    call_user_func_array("extract", [["i" => 1]]);
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Cannot call extract() dynamically\n",
            "Cannot call extract() dynamically\n",
            "Cannot call extract() dynamically\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unresolved_dynamic_function_name_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
$call = "missing";
echo $call();
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined function missing()");
}

#[test]
fn dynamic_function_callee_must_be_string_in_current_subset() {
    let error = runtime_error(
        r#"<?php
$call = 123;
echo $call();
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call dynamic function call: callable expression must evaluate to string, got int"
    );
}

#[test]
fn variable_variables_are_rejected_with_stable_lex_error() {
    let error = lex_error(
        r#"<?php
$name = "value";
$$name = "dynamic";
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported variable variable: variable variables are not implemented"
    );
}

#[test]
fn php_attributes_are_skipped_as_syntax_only_metadata() {
    let execution = run_source(
        r#"<?php
#[Example]
function demo(#[SensitiveParameter] $value) {
    return $value;
}

#[Example]
class Box {
    #[Example]
    public function label(#[SensitiveParameter] $value) {
        return $value;
    }
}

echo demo("ok"), "\n";
$box = new Box();
echo $box->label("box");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ok\nbox");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_comment_with_space_before_bracket_remains_a_comment() {
    let execution = run_source(
        r#"<?php
# [not an attribute]
echo "comment";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "comment");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_require_expressions_return_values_and_share_caller_scope() {
    let root = std::env::temp_dir().join(format!(
        "phpc-include-expression-{}-{}",
        std::process::id(),
        "returns"
    ));
    fs::create_dir_all(&root).expect("create include expression fixture directory");
    let main = root.join("index.php");
    let returner = root.join("returner.php");
    let normal = root.join("normal.php");
    let once = root.join("once.php");
    let required = root.join("required.php");

    fs::write(
        &returner,
        r#"<?php
$side = "changed";
return "from-return";
"#,
    )
    .expect("write returner include file");
    fs::write(
        &normal,
        r#"<?php
$normal_side = "normal-side";
"#,
    )
    .expect("write normal include file");
    fs::write(
        &once,
        r#"<?php
$count = ($count ?? 0) + 1;
return "once-value";
"#,
    )
    .expect("write once include file");
    fs::write(
        &required,
        r#"<?php
$required_side = "required-side";
return "required-value";
"#,
    )
    .expect("write required include file");

    let source = r#"<?php
$path = 'returner.php';
$first = include $path;
echo "first=", $first, ",", $side, "\n";

$normal = include 'normal.php';
echo "normal=", $normal, ",", $normal_side, "\n";

$count = 0;
$once_first = include_once 'once.php';
$once_second = include_once './once.php';
echo "once=", $once_first, ",", $once_second, ",", $count, "\n";

$required = require 'required.php';
$required_once = require_once './required.php';
echo "required=", $required, ",", $required_once, ",", $required_side;
"#;
    fs::write(&main, source).expect("write main include expression file");
    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(
        execution.stdout,
        "first=from-return,changed\nnormal=1,normal-side\nonce=once-value,1,1\nrequired=required-value,1,required-side"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn inline_html_between_php_tags_echoes_as_source_text() {
    let execution = run_source(
        r#"<?php
echo "before";
?>
<div><?php echo "inside"; ?></div>
<?php
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "before<div>inside</div>\nafter");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn require_executes_local_files_with_constant_concat_paths() {
    let root = std::env::temp_dir().join(format!(
        "phpc-require-{}-{}",
        std::process::id(),
        "constant-concat"
    ));
    let wp_includes = root.join("wp-includes");
    fs::create_dir_all(&wp_includes).expect("create require fixture directory");
    let main = root.join("index.php");
    let load = wp_includes.join("load.php");

    fs::write(
        &load,
        r#"<?php
$from = $from . ":load";
function loaded_label() {
    return "loaded";
}
class Loaded {
    public static function name() {
        return "class:" . static::class;
    }
}
"#,
    )
    .expect("write required file");

    let source = format!(
        r#"<?php
const ABSPATH = '{}';
const WPINC = 'wp-includes';
$from = "main";
require ABSPATH . WPINC . '/load.php';
echo loaded_label(), "\n";
echo Loaded::name(), "\n";
echo $from;
"#,
        root.to_string_lossy().replace('\\', "\\\\") + "/"
    );

    fs::write(&main, &source).expect("write main file");
    let execution = run_source_with_source_file(&source, main.display().to_string()).unwrap();

    assert_eq!(execution.stdout, "loaded\nclass:Loaded\nmain:load");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_executes_local_files_in_caller_scope() {
    let root =
        std::env::temp_dir().join(format!("phpc-include-{}-{}", std::process::id(), "local"));
    fs::create_dir_all(&root).expect("create include fixture directory");
    let main = root.join("index.php");
    let lib = root.join("lib.php");

    fs::write(
        &lib,
        r#"<?php
$value = "from-include";
"#,
    )
    .expect("write included file");

    let source = r#"<?php
include 'lib.php';
if (false) {
    include 'missing.php';
}
echo $value;
"#;
    fs::write(&main, source).expect("write main file");
    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(execution.stdout, "from-include");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_once_executes_local_files_only_once() {
    let root = std::env::temp_dir().join(format!(
        "phpc-include-once-{}-{}",
        std::process::id(),
        "dedupe"
    ));
    fs::create_dir_all(&root).expect("create include_once fixture directory");
    let main = root.join("index.php");
    let lib = root.join("lib.php");

    fs::write(
        &lib,
        r#"<?php
$count = ($count ?? 0) + 1;
"#,
    )
    .expect("write included file");

    let source = r#"<?php
$count = 0;
include_once 'lib.php';
include_once './lib.php';
echo $count;
"#;
    fs::write(&main, source).expect("write main file");
    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_once_deduplicates_variable_paths_in_loops() {
    let root = std::env::temp_dir().join(format!(
        "phpc-include-once-{}-{}",
        std::process::id(),
        "loop"
    ));
    fs::create_dir_all(&root).expect("create include_once loop fixture directory");
    let main = root.join("index.php");
    let lib = root.join("mu-plugin.php");

    fs::write(
        &lib,
        r#"<?php
$count = ($count ?? 0) + 1;
"#,
    )
    .expect("write included file");

    let source = r#"<?php
$count = 0;
$plugins = ['mu-plugin.php', './mu-plugin.php'];
foreach ($plugins as $mu_plugin) {
    include_once $mu_plugin;
}
echo $count;
"#;
    fs::write(&main, source).expect("write main file");
    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn top_level_global_declarations_materialize_missing_names_as_null() {
    let execution = run_source(
        r#"<?php
$wp_version = "6.9.4";
global $wp_version, $required_php_version;
echo $required_php_version === null ? "null" : "not-null";
echo "\n";
$required_php_version = "8.3";
echo $wp_version, "\n";
echo $required_php_version, "\n";
if (isset($missing)) {
    echo "missing-set";
} else {
    echo "missing-unset";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "null\n6.9.4\n8.3\nmissing-unset");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn require_executes_relative_to_current_source_file_and_restores_source_mapping() {
    let root = std::env::temp_dir().join(format!(
        "phpc-require-{}-{}",
        std::process::id(),
        "relative-source"
    ));
    fs::create_dir_all(&root).expect("create require fixture directory");
    let main = root.join("index.php");
    let lib = root.join("lib.php");

    fs::write(
        &lib,
        r#"<?php
echo __FILE__, "\n";
$value = "from-lib";
"#,
    )
    .expect("write required file");

    let source = r#"<?php
require 'lib.php';
echo __FILE__, "\n";
echo $value;
"#;
    fs::write(&main, source).expect("write main file");
    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(
        execution.stdout,
        format!("{}\n{}\nfrom-lib", lib.display(), main.display())
    );
}

#[test]
fn require_once_executes_local_files_only_once() {
    let root = std::env::temp_dir().join(format!(
        "phpc-require-once-{}-{}",
        std::process::id(),
        "dedupe"
    ));
    fs::create_dir_all(&root).expect("create require_once fixture directory");
    let main = root.join("index.php");
    let lib = root.join("lib.php");

    fs::write(
        &lib,
        r#"<?php
$count = ($count ?? 0) + 1;
"#,
    )
    .expect("write required file");

    let source = r#"<?php
$count = 0;
require_once 'lib.php';
require_once './lib.php';
echo $count;
"#;
    fs::write(&main, source).expect("write main file");
    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn require_executes_local_files_each_time() {
    let root =
        std::env::temp_dir().join(format!("phpc-require-{}-{}", std::process::id(), "repeat"));
    fs::create_dir_all(&root).expect("create repeated require fixture directory");
    let main = root.join("index.php");
    let lib = root.join("lib.php");

    fs::write(
        &lib,
        r#"<?php
$count = ($count ?? 0) + 1;
"#,
    )
    .expect("write required file");

    let source = r#"<?php
$count = 0;
require 'lib.php';
require 'lib.php';
echo $count;
"#;
    fs::write(&main, source).expect("write main file");
    let execution = run_source_with_source_file(source, main.display().to_string()).unwrap();

    assert_eq!(execution.stdout, "2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn require_reports_current_boundaries() {
    let non_string = runtime_error("<?php\nrequire 123;\n");
    assert_eq!(
        non_string.message,
        "unsupported call require: path must evaluate to a string in the current subset"
    );

    let stream = runtime_error("<?php\nrequire 'https://example.com/file.php';\n");
    assert_eq!(
        stream.message,
        "unsupported call require: stream and URL require paths other than bounded local file:// URLs are not implemented"
    );
}

#[test]
fn emit_ir_rejects_require_until_native_multifile_lowering_exists() {
    for source in [
        "<?php\nrequire 'bootstrap.php';\n",
        "<?php\nrequire_once 'bootstrap.php';\n",
        "<?php\ninclude 'bootstrap.php';\n",
        "<?php\ninclude_once 'bootstrap.php';\n",
        "<?php\n$ok = require 'bootstrap.php';\n",
        "<?php\n$ok = require_once 'bootstrap.php';\n",
        "<?php\n$ok = include 'bootstrap.php';\n",
        "<?php\n$ok = include_once 'bootstrap.php';\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error
                .message
                .contains("include/require lowering rejects multi-file execution"),
            "{}",
            error.message
        );
    }
}

#[test]
fn eval_constructs_are_rejected_with_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
eval('echo "dynamic";');
"#,
            2,
            1,
        ),
        (
            r#"<?php
$result = eval('return 1;');
"#,
            2,
            11,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported eval: eval parsing and caller-scope execution are not implemented"
        );
    }
}

#[test]
fn unsupported_namespace_and_use_forms_are_rejected_with_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
namespace App\Demo {
    echo "blocked";
}
"#,
            2,
            1,
            "unsupported namespace declaration: bracketed namespace blocks are not implemented",
        ),
        (
            r#"<?php
use function App\Demo\make_service;
"#,
            2,
            1,
            "unsupported function use declaration: missing function import metadata, namespace-aware function lookup, alias handling, fallback lookup, and native lowering",
        ),
        (
            r#"<?php
use const App\Demo\VALUE;
"#,
            2,
            1,
            "unsupported const use declaration: missing constant import metadata, namespace-aware constant lookup, alias handling, fallback lookup, and native lowering",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn namespace_qualified_function_names_resolve_before_runtime_lookup() {
    let execution = run_source(
        r#"<?php
namespace App;
function make() {
    return __FUNCTION__;
}
echo namespace\make();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "App\\make");

    let error = runtime_error(
        r#"<?php
namespace App;
App\make();
"#,
    );
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined function App\\App\\make()");
}

#[test]
fn dynamic_undefined_function_calls_are_catchable_errors() {
    let execution = run_source(
        r#"<?php
namespace Foo;
try {
    ('\\bar')();
} catch (\Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Call to undefined function bar()\n");
}

#[test]
fn invalid_array_callbacks_with_missing_zero_one_indices_emit_uncaught_error() {
    let execution = run_source_with_source_file(
        r#"<?php
$callback = [1 => 0, 2 => 0];
$callback();
"#,
        "Zend/tests/dynamic_call/bug63173.php".to_string(),
    )
    .unwrap();

    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught Error: Array callback has to contain indices 0 and 1 in Zend/tests/dynamic_call/bug63173.php:3\nStack trace:\n#0 {main}\n  thrown in Zend/tests/dynamic_call/bug63173.php on line 3"
    );
}

#[test]
fn dynamic_array_callback_errors_use_dynamic_call_messages() {
    let execution = run_source(
        r#"<?php
foreach ([[ "x" ], [null, "bar"], ["stdClass", null], ["a", "b"]] as $callback) {
    try {
        $callback();
    } catch (Error $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Array callback must have exactly two elements\nFirst array member is not a valid class name or object\nSecond array member is not a valid method\nClass \"a\" not found\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_object_callables_use_calling_scope_visibility() {
    let execution = run_source(
        r#"<?php
class ScopedCallableBase {
    private function selected() {
        echo "base:", get_class($this), "\n";
    }

    public function callSelected($method) {
        $this->$method();
        call_user_func([$this, $method]);
        call_user_func_array([$this, $method], []);
    }
}

class ScopedCallableChild extends ScopedCallableBase {
    protected function selected() {
        echo "child\n";
    }
}

$base = new ScopedCallableBase;
$base->callSelected("selected");

$child = new ScopedCallableChild;
$child->callSelected("selected");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base:ScopedCallableBase\nbase:ScopedCallableBase\nbase:ScopedCallableBase\nbase:ScopedCallableChild\nbase:ScopedCallableChild\nbase:ScopedCallableChild\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_static_method_call_names_validate_receiver_then_method_name() {
    let object_method_execution = run_source(
        r#"<?php
$a = new stdClass;
$a::$a();
"#,
    )
    .unwrap();
    assert_eq!(object_method_execution.exit_code, 255);
    assert!(object_method_execution
        .stdout
        .contains("Fatal error: Uncaught Error: Method name must be a string"));

    let int_method_execution = run_source(
        r#"<?php
$a = new stdClass;
$b = 1;
$a::$b();
"#,
    )
    .unwrap();
    assert_eq!(int_method_execution.exit_code, 255);
    assert!(int_method_execution
        .stdout
        .contains("Fatal error: Uncaught Error: Method name must be a string"));

    let execution = run_source_with_source_file(
        r#"<?php
$a::$b();
"#,
        "dynamic_call_004.php".to_string(),
    )
    .unwrap();
    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stdout,
        "Warning: Undefined variable $a in dynamic_call_004.php on line 2

Fatal error: Uncaught Error: Class name must be a valid object or a string in dynamic_call_004.php:2
Stack trace:
#0 {main}
  thrown in dynamic_call_004.php on line 2"
    );
}
#[test]
fn dynamic_static_method_string_calls_are_dispatched() {
    let execution = run_source(
        r#"<?php
class DynamicTarget {
    public static function ping() {
        echo "ping\n";
    }

    public static function withArgs($left, $middle, $right) {
        $length = printf("args:%s,%s,%s\n", $left, $middle, $right);
        echo "length=", $length, "\n";
    }
}

$callback = "DynamicTarget::ping";
$callback();

$callback = "dynamictarget::PING";
$callback();

$callback = "DynamicTarget::withArgs";
$callback("left", "middle", "right");

$args = ["left", "middle", "right"];
$callback(...$args);

$callback = "DynamicTarget::missing";
try {
    $callback();
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}

$callback = "MissingDynamicTarget::ping";
try {
    $callback();
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ping\nping\nargs:left,middle,right\nlength=23\nargs:left,middle,right\nlength=23\nCall to undefined method DynamicTarget::missing()\nClass \"MissingDynamicTarget\" not found\n"
    );
}

#[test]
fn static_object_array_callbacks_do_not_bind_this() {
    let execution = run_source(
        r#"<?php
class StaticCallbackTarget {
    public static function show($value = null) {
        var_dump($this);
    }
}

try {
    array_map([new StaticCallbackTarget, "show"], [1]);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    call_user_func([new StaticCallbackTarget, "show"]);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Using $this when not in object context\nUsing $this when not in object context\n"
    );
}

#[test]
fn fully_qualified_function_calls_use_exact_global_lookup() {
    let execution = run_source(
        r#"<?php
namespace App;
function make() {
    return __FUNCTION__;
}
echo \strlen("abc"), "|", \App\make();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3|App\\make");
}

#[test]
fn bare_global_constants_resolve_runtime_defined_and_builtin_values() {
    let execution = run_source(
        r#"<?php
define("APP_NAME", "compiler");
define("APP_VERSION", 2);
echo APP_NAME, "|", APP_VERSION + 3, "\n";
echo ARRAY_FILTER_USE_KEY, "|", ARRAY_FILTER_USE_BOTH, "\n";
echo PHP_VERSION_ID, "|", PHP_VERSION_ID >= 80000, "\n";

$items = ["name" => "Ada", "nested" => ["x" => 1]];
define("APP_ITEMS", $items);
$copy = APP_ITEMS;
$copy["name"] = "changed";
echo APP_ITEMS["name"], "|", APP_ITEMS["nested"]["x"], "|", $copy["name"], "\n";

function read_constant_inside_function() {
    define("FUNCTION_CONSTANT", "inside");
    return APP_NAME . ":" . FUNCTION_CONSTANT;
}

echo read_constant_inside_function(), "\n";
$call = "define";
$call("DYNAMIC_CONSTANT", "dynamic");
echo DYNAMIC_CONSTANT, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "compiler|5\n2|1\n80300|1\nAda|1|changed\ncompiler:inside\ndynamic\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn undefined_bare_global_constants_have_stable_runtime_errors() {
    let cases = [(
        r#"<?php
echo array_filter([], "strlen", CUSTOM_FILTER_MODE);
"#,
        2,
        33,
        "CUSTOM_FILTER_MODE",
    )];

    for (source, line, column, name) in cases {
        let error = runtime_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, format!("undefined constant {name}"));
    }
}

#[test]
fn constant_builtin_resolves_the_current_builtin_constant_slice() {
    let execution = run_source(
        r#"<?php
echo constant("ARRAY_FILTER_USE_KEY"), "|", constant("ARRAY_FILTER_USE_BOTH"), "\n";
echo constant("PHP_VERSION_ID"), "|", constant("PHP_VERSION_ID") >= 80000, "\n";
echo defined("PHP_VERSION"), "|", PHP_VERSION === constant("PHP_VERSION"), "\n";
echo defined("PHP_EOL"), "|", constant("PHP_EOL") === "\n", "|", PHP_EOL, "after", "\n";
define("Sodium\\CRYPTO_AUTH_BYTES", 32);
echo constant("\\Sodium\\CRYPTO_AUTH_BYTES"), "\n";
$name = "ARRAY_FILTER_USE_KEY";
echo constant($name), "\n";
$call = "constant";
echo $call("ARRAY_FILTER_USE_BOTH"), "\n";

function keep_named_key($key) {
    return $key === "name";
}

$items = ["name" => "Ada", "other" => "Lin"];
$filtered = array_filter($items, "keep_named_key", constant("ARRAY_FILTER_USE_KEY"));
print_r(array_keys($filtered));
echo count($filtered), "|", $filtered["name"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2|1\n80300|1\n1|1\n1|1|\nafter\n32\n2\n1\nArray\n(\n    [0] => name\n)\n1|Ada\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn constant_builtin_resolves_declared_class_constant_string_names() {
    let execution = run_source(
        r#"<?php
class ParagonIE_Sodium_Compat {
    const LIBRARY_VERSION_MAJOR = 9;
    public const LIBRARY_VERSION_MINOR = 99;
}
class Child_Compat extends ParagonIE_Sodium_Compat {}

$constant = "LIBRARY_VERSION_MAJOR";
echo defined("ParagonIE_Sodium_Compat::$constant") ? "1" : "0";
echo "|", constant("ParagonIE_Sodium_Compat::$constant"), "\n";
echo defined("\\ParagonIE_Sodium_Compat::LIBRARY_VERSION_MINOR") ? "1" : "0";
echo "|", constant("\\ParagonIE_Sodium_Compat::LIBRARY_VERSION_MINOR"), "\n";
echo defined("Child_Compat::LIBRARY_VERSION_MAJOR") ? "1" : "0";
echo "|", constant("Child_Compat::LIBRARY_VERSION_MAJOR"), "\n";
echo defined("ParagonIE_Sodium_Compat::MISSING") ? "1" : "0";
echo "|", defined("Missing_Compat::LIBRARY_VERSION_MAJOR") ? "1" : "0", "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|9\n1|99\n1|9\n0|0\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn defined_class_constant_string_names_report_only_public_constants() {
    let execution = run_source(
        r#"<?php
class SecretBox {
    private const SECRET = "secret";
    protected const HIDDEN = "hidden";
    public const OPEN = "open";
}
echo defined("SecretBox::SECRET") ? "1" : "0";
echo "|", defined("SecretBox::HIDDEN") ? "1" : "0";
echo "|", defined("SecretBox::OPEN") ? "1" : "0", "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|0|1\n");
    assert_eq!(execution.exit_code, 0);

    let private_error = runtime_error(
        r#"<?php
class SecretBox {
    private const SECRET = "secret";
}
echo constant("SecretBox::SECRET");
"#,
    );

    assert_eq!(private_error.line, 5);
    assert_eq!(private_error.column, 6);
    assert_eq!(
        private_error.message,
        "unsupported call SecretBox::SECRET: private class constant is not visible from the current class context"
    );
}

#[test]
fn constant_builtin_rejects_unknown_constant_names() {
    let error = runtime_error(
        r#"<?php
echo constant("PHP_OS");
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call constant(): constant PHP_OS is not defined in the current runtime-defined or built-in constant subset"
    );

    let class_constant = runtime_error(
        r#"<?php
class Box {}
echo constant("Box::MISSING");
"#,
    );

    assert_eq!(class_constant.line, 3);
    assert_eq!(class_constant.column, 6);
    assert_eq!(class_constant.message, "undefined constant Box::MISSING");
}

#[test]
fn error_control_operator_evaluates_operand_without_suppression() {
    let execution = run_source(
        r#"<?php
$c = 5;
echo @($c & -1), "\n";
echo (int) @($c & -1), "\n";
echo @"ok", "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "5\n5\nok\n");
    assert_eq!(execution.exit_code, 0);

    let error = runtime_error(
        r#"<?php
echo @$missing;
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn constant_builtin_requires_string_names() {
    let error = runtime_error(
        r#"<?php
echo constant(42);
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call constant(): name argument must be string in the current subset, got int"
    );
}

#[test]
fn defined_builtin_introspects_current_constant_table() {
    let execution = run_source(
        r#"<?php
echo defined("ARRAY_FILTER_USE_KEY"), "|", defined("ARRAY_FILTER_USE_BOTH"), "\n";
echo defined("PHP_VERSION_ID"), "|", defined("PHP_VERSION"), "\n";
echo defined("APP_NAME"), "|", defined("MISSING_CONST"), "\n";
define("APP_NAME", "compiler");
define("Sodium\\RUNTIME_CONST", 99);
echo defined("APP_NAME"), "|", defined("MISSING_CONST"), "\n";
$call = "defined";
echo $call("APP_NAME"), "|", $call("MISSING_CONST"), "\n";
echo defined("\\PHP_VERSION_ID"), "|", defined("\\Sodium\\CRYPTO_AUTH_BYTES"), "|", defined("Sodium\\CRYPTO_AUTH_BYTES"), "\n";
echo defined("\\Sodium\\RUNTIME_CONST"), "|", constant("\\Sodium\\RUNTIME_CONST"), "\n";
$qualified = "\\Sodium\\CRYPTO_AUTH_BYTES";
echo $call($qualified), "\n";

function check_defined_inside_function() {
    define("INSIDE_DEFINED", 1);
    return defined("INSIDE_DEFINED") . ":" . defined("APP_NAME");
}

echo check_defined_inside_function(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|1\n1|1\n|\n1|\n1|\n1||\n1|99\n\n1:1\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_defined_constants_exports_builtin_and_runtime_constant_keys() {
    let execution = run_source(
        r#"<?php
$before = get_defined_constants();
echo isset($before["PHP_URL_SCHEME"]) ? $before["PHP_URL_SCHEME"] : "missing";
echo "|", isset($before["PHP_URL_FRAGMENT"]) ? $before["PHP_URL_FRAGMENT"] : "missing", "\n";
echo gettype(get_defined_constants(true)), "|", gettype($before), "\n";
define("USER_CONSTANT", "test");
$after = get_defined_constants(false);
echo array_key_exists("USER_CONSTANT", $after) ? $after["USER_CONSTANT"] : "missing";
echo "|", count($after) === count($before) + 1 ? "grew" : "stale";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|7\narray|array\ntest|grew");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_defined_constants_exports_supported_builtin_constant_registry() {
    let execution = run_source(
        r#"<?php
$constants = get_defined_constants();
$names = [
    "INPUT_POST",
    "INPUT_GET",
    "INPUT_COOKIE",
    "FILTER_VALIDATE_INT",
    "FILTER_VALIDATE_BOOLEAN",
    "FILTER_VALIDATE_BOOL",
    "FILTER_VALIDATE_FLOAT",
    "FILTER_VALIDATE_REGEXP",
    "FILTER_VALIDATE_DOMAIN",
    "FILTER_VALIDATE_URL",
    "FILTER_VALIDATE_EMAIL",
    "FILTER_VALIDATE_IP",
    "FILTER_VALIDATE_MAC",
    "FILTER_DEFAULT",
    "FILTER_UNSAFE_RAW",
    "FILTER_SANITIZE_STRING",
    "FILTER_SANITIZE_STRIPPED",
    "FILTER_SANITIZE_ENCODED",
    "FILTER_SANITIZE_SPECIAL_CHARS",
    "FILTER_SANITIZE_FULL_SPECIAL_CHARS",
    "FILTER_SANITIZE_EMAIL",
    "FILTER_SANITIZE_URL",
    "FILTER_SANITIZE_NUMBER_INT",
    "FILTER_SANITIZE_NUMBER_FLOAT",
    "FILTER_SANITIZE_ADD_SLASHES",
    "FILTER_CALLBACK",
    "FILTER_REQUIRE_ARRAY",
    "FILTER_FORCE_ARRAY",
    "FILTER_NULL_ON_FAILURE",
    "COUNT_NORMAL",
    "COUNT_RECURSIVE",
];

foreach ($names as $name) {
    echo $name, ":";
    echo defined($name) ? "defined" : "missing";
    echo ":";
    echo array_key_exists($name, $constants) ? "listed" : "unlisted";
    echo ":";
    echo $constants[$name] === constant($name) ? "same" : "different";
    echo "\n";
}
"#,
    )
    .unwrap();

    for line in execution.stdout.lines() {
        assert!(
            line.ends_with(":defined:listed:same"),
            "unexpected constant registry line: {line}"
        );
    }
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn defined_builtin_requires_string_names_and_supported_names() {
    let non_string = runtime_error(
        r#"<?php
echo defined(42);
"#,
    );
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 6);
    assert_eq!(
        non_string.message,
        "unsupported call defined(): name argument must be string in the current subset, got int"
    );

    let bad_name = runtime_error(
        r#"<?php
echo defined("123BAD");
"#,
    );
    assert_eq!(bad_name.line, 2);
    assert_eq!(bad_name.column, 6);
    assert_eq!(
        bad_name.message,
        "unsupported call defined(): constant name must be a non-empty supported identifier or qualified name in the current subset, got 123BAD"
    );

    for source in [
        r#"<?php
echo defined("");
"#,
        r#"<?php
echo defined("\\");
"#,
        r#"<?php
echo defined("Sodium\\");
"#,
        r#"<?php
echo defined("Sodium\\\\CRYPTO_AUTH_BYTES");
"#,
        r#"<?php
echo defined("\\123BAD");
"#,
    ] {
        let malformed = runtime_error(source);
        assert_eq!(malformed.line, 2);
        assert_eq!(malformed.column, 6);
        assert!(
            malformed.message.contains(
                "constant name must be a non-empty supported identifier or qualified name"
            ),
            "{}",
            malformed.message
        );
    }
}

#[test]
fn double_quoted_strings_interpolate_simple_variables_for_constant_name_builtins() {
    let execution = run_source(
        r#"<?php
$constant = "RUNTIME";
define("APP_RUNTIME", "ok");
echo defined("APP_$constant") ? "1" : "0";
echo "|", constant("APP_$constant"), "\n";
$constant = "MISSING";
echo defined("APP_$constant") ? "1" : "0", "\n";
echo "literal:\$constant", "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|ok\n0\nliteral:$constant\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn double_quoted_strings_interpolate_simple_braced_variables() {
    let execution = run_source(
        r#"<?php
$name = "Ada";
$suffix = "RUNTIME";
$term_count = 7;
define("APP_RUNTIME", "ok");
echo "hello {$name}", "\n";
echo defined("APP_{$suffix}") ? "1" : "0";
echo "|", constant("APP_{$suffix}"), "\n";
echo "{{$term_count}}", "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello Ada\n1|ok\n{7}\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn double_quoted_strings_interpolate_current_array_offsets_and_object_properties() {
    let execution = run_source(
        r#"<?php
$attributes = ["textAlign" => "center", "layout" => ["columns" => 3]];
$key = "textAlign";
class Partial {
    public $id;
    public $context;
}
$partial = new Partial();
$partial->id = "header";
$partial->context = ["displayLayout" => ["columns" => 4]];
echo "has-text-align-{$attributes['textAlign']}";
echo "|has-text-align-{$attributes[$key]}";
echo "|has-text-align-$attributes[textAlign]";
echo "|customize_partial_render_{$partial->id}";
echo "|columns-{$attributes['layout']['columns']}";
echo "|columns-{$partial->context['displayLayout']['columns']}";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "has-text-align-center|has-text-align-center|has-text-align-center|customize_partial_render_header|columns-3|columns-4"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn non_variable_dollar_runs_in_interpolated_strings_are_literal() {
    let execution = run_source(
        r#"<?php
echo "$$$$$$!!!!@@@@@@@ ABCDEF !!!***", "\n";
echo "cost $$";
$byte = chr(128);
echo "\n", bin2hex("$byte");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "$$$$$$!!!!@@@@@@@ ABCDEF !!!***\ncost $$\n80"
    );
    assert_eq!(execution.exit_code, 0);

    let variable_variable = lex_error(
        r#"<?php
$name = "value";
echo "$$name";
"#,
    );
    assert_eq!(variable_variable.line, 3);
    assert_eq!(variable_variable.column, 6);
    assert_eq!(
        variable_variable.message,
        "unsupported string interpolation: only simple $name, {$name}, array offsets, and object properties in double-quoted strings are implemented; ${...}, dynamic properties, static properties, arbitrary expressions, and complex interpolation are not implemented"
    );
}

#[test]
fn remaining_complex_string_interpolation_forms_keep_named_boundaries() {
    let dollar_brace = lex_error(
        r#"<?php
$name = "value";
echo "${name}";
"#,
    );
    assert_eq!(dollar_brace.line, 3);
    assert_eq!(dollar_brace.column, 6);
    assert_eq!(
        dollar_brace.message,
        "unsupported string interpolation: only simple $name, {$name}, array offsets, and object properties in double-quoted strings are implemented; ${...}, dynamic properties, static properties, arbitrary expressions, and complex interpolation are not implemented"
    );

    let dynamic_property = lex_error(
        r#"<?php
class Partial {
    public $id;
}
$partial = new Partial();
$property = "id";
echo "customize_partial_render_{$partial->{$property}}";
"#,
    );
    assert_eq!(dynamic_property.line, 7);
    assert_eq!(dynamic_property.column, 6);
    assert_eq!(
        dynamic_property.message,
        "unsupported string interpolation: only simple $name, {$name}, array offsets, and object properties in double-quoted strings are implemented; ${...}, dynamic properties, static properties, arbitrary expressions, and complex interpolation are not implemented"
    );
}

#[test]
fn undefined_variables_in_string_interpolation_use_current_runtime_error() {
    let error = runtime_error(
        r#"<?php
echo "APP_$constant";
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$constant'");
}

#[test]
fn undefined_variables_in_braced_string_interpolation_use_current_runtime_error() {
    let error = runtime_error(
        r#"<?php
echo "APP_{$constant}";
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$constant'");
}

#[test]
fn top_level_const_declarations_populate_constant_table() {
    let execution = run_source(
        r#"<?php
const APP_NAME = "compiler";
CONST APP_VERSION = 2;
const APP_SCALE = 1 + 2 * 3;
const APP_ITEMS = ["name" => "Ada", "count" => 2, "nested" => ["x" => 1]];
echo APP_NAME, "|", APP_VERSION, "|", APP_SCALE, "\n";
echo constant("APP_NAME"), "|", defined("APP_ITEMS"), "|", defined("MISSING_CONST"), "\n";
$copy = APP_ITEMS;
$copy["name"] = "changed";
echo count($copy), "|", $copy["name"], "|", APP_ITEMS["name"], "|", APP_ITEMS["nested"]["x"], "\n";
function read_declared_const() {
    return APP_NAME . ":" . APP_VERSION;
}
echo read_declared_const(), "\n";
$name = "APP_NAME";
echo constant($name), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "compiler|2|7\ncompiler|1|\n3|changed|Ada|1\ncompiler:2\ncompiler\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn grouped_top_level_const_declarations_execute_left_to_right() {
    let execution = run_source(
        r#"<?php
const APP_NAME = "compiler", APP_VERSION = 2, APP_SCALE = 1 + 2 * 3;
CONST APP_FLAGS = ["env" => "dev", "nested" => ["x" => 1]], APP_EMPTY = [];
echo APP_NAME, "|", APP_VERSION, "|", APP_SCALE, "|", defined("APP_EMPTY"), "\n";
$copy = APP_FLAGS;
$copy["env"] = "prod";
echo $copy["env"], "|", APP_FLAGS["env"], "|", APP_FLAGS["nested"]["x"], "\n";
function read_grouped_const() {
    return APP_NAME . ":" . APP_VERSION . ":" . APP_FLAGS["nested"]["x"];
}
echo read_grouped_const(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "compiler|2|7|1\nprod|dev|1\ncompiler:2:1\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn top_level_const_declaration_values_can_reference_prior_constants() {
    let execution = run_source(
        r#"<?php
define("RUNTIME_BASE", 3);
const FROM_DEFINE = RUNTIME_BASE + 1;
const BASE = "compiler";
const VERSION = 2, DOUBLE_VERSION = VERSION * 2, LABEL = BASE . ":" . DOUBLE_VERSION;
const FILTER_MODE = ARRAY_FILTER_USE_BOTH;
const ITEMS = [BASE => LABEL, "mode" => FILTER_MODE, "key-mode" => ARRAY_FILTER_USE_KEY, "from-define" => FROM_DEFINE];
echo LABEL, "|", FILTER_MODE, "|", ITEMS["compiler"], "|", ITEMS["mode"], "|", ITEMS["key-mode"], "|", ITEMS["from-define"], "\n";
function read_referenced_const() {
    return LABEL . ":" . ARRAY_FILTER_USE_KEY;
}
echo read_referenced_const(), "\n";
$name = "DOUBLE_VERSION";
echo constant($name), "|", FROM_DEFINE, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "compiler:4|1|compiler:4|1|2|4\ncompiler:4:2\n4|4\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn forward_const_declaration_references_have_stable_runtime_diagnostics() {
    let error = runtime_error(
        r#"<?php
const FORWARD = LATER, LATER = "done";
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 17);
    assert_eq!(error.message, "undefined constant LATER");
}

#[test]
fn duplicate_top_level_const_declarations_have_stable_diagnostics() {
    let duplicate = runtime_error(
        r#"<?php
const APP_NAME = "compiler";
const APP_NAME = "again";
"#,
    );
    assert_eq!(duplicate.line, 3);
    assert_eq!(duplicate.column, 1);
    assert_eq!(duplicate.message, "constant APP_NAME is already defined");

    let builtin = runtime_error(
        r#"<?php
const PHP_VERSION_ID = 4;
"#,
    );
    assert_eq!(builtin.line, 2);
    assert_eq!(builtin.column, 1);
    assert_eq!(
        builtin.message,
        "constant PHP_VERSION_ID is already defined"
    );

    let grouped = runtime_error(
        r#"<?php
const APP_NAME = "compiler", APP_VERSION = 1, APP_NAME = "again";
"#,
    );
    assert_eq!(grouped.line, 2);
    assert_eq!(grouped.column, 47);
    assert_eq!(grouped.message, "constant APP_NAME is already defined");
}

#[test]
fn unsupported_const_declaration_forms_have_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
if (true) {
    const INSIDE = 1;
}
"#,
            3,
            5,
            "unsupported const declaration: only top-level constant declarations are implemented",
        ),
        (
            r#"<?php
const APP_NAME = $name;
"#,
            2,
            18,
            "const declaration values only support constant expressions in the current subset",
        ),
        (
            r#"<?php
class Box {}
const BOX = new Box();
"#,
            3,
            13,
            "const declaration values only support constant expressions in the current subset",
        ),
        (
            r#"<?php
const APP\NAME = 1;
"#,
            2,
            10,
            "unsupported const declaration: namespace-qualified constant declarations are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn define_builtin_populates_runtime_constant_table() {
    let execution = run_source(
        r#"<?php
define("APP_NAME", "compiler");
echo define("APP_VERSION", 1), "\n";
echo constant("APP_NAME"), "|", constant("APP_VERSION"), "\n";

$items = ["name" => "Ada", "count" => 2, "nested" => ["x" => 1]];
define("APP_ITEMS", $items);
$copy = constant("APP_ITEMS");
$copy["name"] = "changed";
$again = constant("APP_ITEMS");
echo count($copy), "|", $copy["name"], "|", $again["name"], "|", $again["nested"]["x"], "\n";

function constant_scope() {
    define("INSIDE_FUNCTION", "inside");
    return constant("APP_NAME") . ":" . constant("INSIDE_FUNCTION");
}

echo constant_scope(), "\n";
$call = "define";
echo $call("DYNAMIC_NAME", "dynamic"), "\n";
echo constant("DYNAMIC_NAME"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1\ncompiler|1\n3|changed|Ada|1\ncompiler:inside\n1\ndynamic\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn duplicate_runtime_constant_definitions_have_stable_diagnostics() {
    let error = runtime_error(
        r#"<?php
define("APP_NAME", "compiler");
define("APP_NAME", "again");
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "constant APP_NAME is already defined");
}

#[test]
fn define_rejects_builtin_constant_redefinition() {
    let error = runtime_error(
        r#"<?php
define("PHP_VERSION_ID", 4);
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "constant PHP_VERSION_ID is already defined");
}

#[test]
fn define_requires_string_names_and_supported_values() {
    let non_string = runtime_error(
        r#"<?php
define(42, "bad");
"#,
    );
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call define(): name argument must be string in the current subset, got int"
    );

    let bad_name = runtime_error(
        r#"<?php
define("123BAD", "bad");
"#,
    );
    assert_eq!(bad_name.line, 2);
    assert_eq!(bad_name.column, 1);
    assert_eq!(
        bad_name.message,
        "unsupported call define(): constant name must be a non-empty supported identifier or qualified name in the current subset, got 123BAD"
    );

    let unsupported_value = runtime_error(
        r#"<?php
class Box {}
define("BOX", new Box());
"#,
    );
    assert_eq!(unsupported_value.line, 3);
    assert_eq!(unsupported_value.column, 1);
    assert_eq!(
        unsupported_value.message,
        "unsupported call define(): value must be null, bool, int, float, string, or array values in the current subset, got object"
    );
}

#[test]
fn define_rejects_case_insensitive_legacy_flag() {
    let error = runtime_error(
        r#"<?php
define("APP_NAME", "compiler", true);
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call define(): case-insensitive constant definitions are not implemented; pass exactly two arguments in the current subset"
    );
}

#[test]
fn emit_ir_rejects_constant_lookup_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho constant(\"ARRAY_FILTER_USE_KEY\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("global-constant lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_define_until_user_constant_lowering_exists() {
    let error = emit_ir_source("<?php\ndefine(\"APP_NAME\", \"compiler\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("global-constant lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_unsupported_defined_names_until_native_constant_tables_exist() {
    let error = emit_ir_source("<?php\necho defined(\"123BAD\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("global-constant lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_const_declarations_until_native_lowering_exists() {
    let error =
        emit_ir_source("<?php\nconst APP_NAME = \"compiler\", APP_VERSION = 2;\necho APP_NAME;\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("global-constant lowering"),
        "{}",
        error.message
    );
}
