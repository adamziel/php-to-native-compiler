use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;
use std::path::Path;
use std::process::Command;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn system_php_available() -> bool {
    Command::new("php").arg("-v").output().is_ok()
}

#[test]
fn system_php_preserves_copied_arrayaccess_buckets_in_stored_call_user_func_array_args() {
    if !system_php_available() {
        return;
    }

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tests/probes/milestone1673/stored_call_user_func_array_arrayaccess_bucket_cow.php",
    );
    let expected = std::fs::read_to_string(fixture.with_extension("stdout"))
        .expect("read stored call_user_func_array ArrayAccess bucket probe expectation");
    let output = Command::new("php")
        .arg(&fixture)
        .output()
        .expect("run system PHP stored call_user_func_array ArrayAccess bucket probe");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn call_user_func_array_preserves_copied_arrayaccess_buckets_in_stored_args() {
    let source = include_str!(
        "../../tests/probes/milestone1673/stored_call_user_func_array_arrayaccess_bucket_cow.php"
    );
    let expected = include_str!(
        "../../tests/probes/milestone1673/stored_call_user_func_array_arrayaccess_bucket_cow.stdout"
    );
    let execution = run_source(source).unwrap();

    assert_eq!(execution.stdout, expected);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_invokes_current_string_callable_subset() {
    let execution = run_source(
        r#"<?php
function greet($name) {
    return "hi " . $name;
}
echo call_user_func("greet", "Ada"), "\n";
echo call_user_func("str_replace", " ", "_", "hello world"), "\n";
$call = "call_user_func";
echo $call("strlen", "four");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hi Ada\nhello_world\n4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_dispatches_missing_static_methods_through_callstatic() {
    let execution = run_source(
        r#"<?php
class StaticMagicCallback {
    public static function __callStatic($name, $args) {
        echo $name, ":", count($args), "\n";
        return $name . "-" . count($args);
    }
}

echo call_user_func("StaticMagicCallback::Two", "A", "B"), "\n";
echo call_user_func(array("StaticMagicCallback", "Three"), NULL, 0, false), "\n";
echo call_user_func_array("StaticMagicCallback::Arrayed", array("X", "Y")), "\n";
echo call_user_func_array(array("StaticMagicCallback", "Pair"), array("L", "R")), "\n";
echo StaticMagicCallback::Direct(1, 2, 3);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Two:2\nTwo-2\nThree:3\nThree-3\nArrayed:2\nArrayed-2\nPair:2\nPair-2\nDirect:3\nDirect-3"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_builtin_callback_expands_integer_unpacked_arguments() {
    let execution = run_source(
        r#"<?php
$strlen_args = ["four"];
echo call_user_func("strlen", ...$strlen_args), "
";

$replace_args = [" ", "_", "hello world"];
echo call_user_func("str_replace", ...$replace_args), "
";

$dynamic = "call_user_func";
echo $dynamic("count", ...[[1, 2, 3]]);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "4
hello_world
3"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_invokes_current_string_callable_and_positional_array_subset() {
    let execution = run_source(
        r#"<?php
function join_names($first, $second = "Grace") {
    return $first . "+" . $second;
}
echo call_user_func_array("join_names", array("Ada", "Linus")), "\n";
echo call_user_func_array("str_replace", array(" ", "_", "hello world")), "\n";
$call = "call_user_func_array";
echo $call("strlen", array("four"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada+Linus\nhello_world\n4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_preserves_named_variadic_argument_keys() {
    let execution = run_source(
        r#"<?php
$with_defaults = function ($a = "a", $b = "b", $c = "c") {
    echo "a=$a,b=$b,c=$c
";
};
$variadic = function (...$args) {
    foreach ($args as $key => $value) {
        echo $key, "=", $value, ";";
    }
};

call_user_func_array($with_defaults, array("A", "c" => "C"));
call_user_func_array($variadic, array("A", "c" => "C"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "a=A,b=b,c=C
0=A;c=C;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_named_arguments_for_metadata_backed_builtins() {
    let execution = run_source(
        r#"<?php
$strlen = array("string" => "four");
echo call_user_func_array("strlen", $strlen), "\n";

$substr = array("string" => "abcdef", "length" => 3, "offset" => 1);
echo call_user_func_array("substr", $substr), "\n";

$count = array("value" => array("a" => 1, "b" => 2));
echo call_user_func_array("count", $count);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "4\nbcd\n2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_named_reference_arguments_for_builtin_callbacks() {
    let execution = run_source(
        r#"<?php
$values = array(10, 20, 30);
echo call_user_func_array("array_pop", array("array" => &$values)), "|";
echo count($values), "|", $values[0], ":", $values[1], "
";

$assoc = array(2 => "two", 1 => "one");
echo call_user_func_array("ksort", array("array" => &$assoc, "flags" => 1)) ? "sorted" : "not";
echo "|";
foreach ($assoc as $key => $value) {
    echo $key, "=", $value, ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "30|2|10:20
sorted|1=one;2=two;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn closure_values_invoke_directly_and_through_callback_dispatch() {
    let execution = run_source(
        r#"<?php
$prefix = "wp";
$callback = function ($hook, $priority = 10) use ($prefix) {
    return $prefix . ":" . $hook . ":" . $priority;
};
$prefix = "changed";

echo $callback("init"), "\n";
echo call_user_func($callback, "plugins_loaded", 5), "\n";
echo call_user_func_array($callback, array("save_post", 20));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "wp:init:10\nwp:plugins_loaded:5\nwp:save_post:20"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn direct_closure_invocation_binds_reference_parameters() {
    let execution = run_source(
        r#"<?php
$counter = 0;
$option = "autoload";
$items = array("payload" => array("slot" => "start"));
$callback = function (&$value, $suffix) use (&$counter) {
    $counter = $counter + 1;
    $value = $value . ":" . $suffix . ":" . $counter;
    return $value;
};

echo $callback($option, "direct"), "|", $option, "|", $counter, "\n";
echo $callback($items["payload"]["slot"], "slot"), "|", $items["payload"]["slot"], "|", $counter;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:direct:1|autoload:direct:1|1\nstart:slot:2|start:slot:2|2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_closure_reference_parameters() {
    let execution = run_source(
        r#"<?php
$counter = 0;
$option = "autoload";
$callback = function (&$value, $suffix) use (&$counter) {
    $counter = $counter + 1;
    $value = $value . ":" . $suffix . ":" . $counter;
    return $value;
};

echo call_user_func_array($callback, array(&$option, "closure")), "|", $option, "|", $counter, "\n";
echo call_user_func_array($callback, array(&$option, "again")), "|", $option, "|", $counter;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:closure:1|autoload:closure:1|1\nautoload:closure:1:again:2|autoload:closure:1:again:2|2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_invokes_reference_parameters_by_value_with_warning() {
    let execution = run_source(
        r#"<?php
function milestone1630_warning($errno, $errstr) {
    echo "warning:" . $errno . ":" . (str_contains($errstr, "must be passed by reference") ? "ref" : "other") . "\n";
    return true;
}

function milestone1630_mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

set_error_handler("milestone1630_warning", E_WARNING);
$option = "autoload";
$items = array("payload" => array("slot" => "start"));
echo call_user_func("milestone1630_mark", $option, "direct"), "|", $option, "\n";
echo call_user_func("milestone1630_mark", $items["payload"]["slot"], "slot"), "|", $items["payload"]["slot"], "\n";
$counter = 0;
$callback = function (&$value, $suffix) use (&$counter) {
    $counter = $counter + 1;
    $value = $value . ":" . $suffix . ":" . $counter;
    return $value;
};
echo call_user_func($callback, $option, "closure"), "|", $option, "|", $counter;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "warning:2:ref\nautoload:direct|autoload\nwarning:2:ref\nstart:slot|start\nwarning:2:ref\nautoload:closure:1|autoload|1"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_callables_invoke_reference_parameters_by_value_with_warning() {
    let execution = run_source(
        r#"<?php
function milestone1636_warning($errno, $errstr) {
    echo "warning:" . $errno . ":" . (str_contains($errstr, "must be passed by reference") ? "ref" : "other") . "\n";
    return true;
}

class Milestone1636_Filter {
    public $seen = "seed";

    public function mark(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        $this->seen = $this->seen . ":" . $suffix;
        return $value . ":" . $this->seen;
    }

    public static function tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

set_error_handler("milestone1636_warning", E_WARNING);
$filter = new Milestone1636_Filter();
$option = "autoload";
$items = array("payload" => array("slot" => "start"));
echo call_user_func(array($filter, "mark"), $option, "object"), "|", $option, "|", $filter->seen, "\n";
echo call_user_func(array("Milestone1636_Filter", "tag"), $items["payload"]["slot"], "static"), "|", $items["payload"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "warning:2:ref\nautoload:object:seed:object|autoload|seed:object\nwarning:2:ref\nstart:static|start"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn closures_capture_alias_backed_array_and_property_slots_by_reference() {
    let execution = run_source(
        r#"<?php
$_REQUEST["payload"] = array("slot" => "start");
$payload =& $_REQUEST["payload"];
$callback = function ($suffix) use (&$payload) {
    $payload["slot"] = $payload["slot"] . ":" . $suffix;
    return $payload["slot"];
};

echo $callback("direct"), "|", $_REQUEST["payload"]["slot"], "\n";
echo call_user_func($callback, "call"), "|", $_REQUEST["payload"]["slot"], "\n";
echo call_user_func_array($callback, array("array")), "|", $_REQUEST["payload"]["slot"], "\n";
$reflected = new ReflectionFunction($callback);
echo $reflected->invoke("reflect"), "|", $_REQUEST["payload"]["slot"], "\n";

class Milestone1642_Box {
    public $items = array("slot" => "box");
}

$box = new Milestone1642_Box();
$item =& $box->items["slot"];
$propertyCallback = function ($suffix) use (&$item) {
    $item = $item . ":" . $suffix;
    return $item;
};

echo $propertyCallback("property"), "|", $box->items["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "start:direct|start:direct\nstart:direct:call|start:direct:call\nstart:direct:call:array|start:direct:call:array\nstart:direct:call:array:reflect|start:direct:call:array:reflect\nbox:property|box:property"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_literal_reference_arguments_for_user_callbacks() {
    let execution = run_source(
        r#"<?php
function update_option_like(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
$callback = "update_option_like";
echo call_user_func_array($callback, array(&$option, "cache")), "\n";
echo $option;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "autoload:cache\nautoload:cache");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_literal_reference_arguments_for_object_array_callbacks() {
    let execution = run_source(
        r#"<?php
class OptionFilter {
    public function update(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

$filter = new OptionFilter();
$option = "autoload";
echo call_user_func_array(array($filter, "update"), array(&$option, "object-cache")), "\n";
echo $option;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:object-cache\nautoload:object-cache"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_writes_back_object_property_array_reference_arguments() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];
}

class CacheFilter {
    public function mark(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function tag_cache(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
call_user_func_array("tag_cache", array(&$cache->cache["options"]["alloptions"], "function"));
$filter = new CacheFilter();
echo call_user_func_array(array($filter, "mark"), array(&$cache->cache["options"]["alloptions"], "method")), "\n";
echo $cache->cache["options"]["alloptions"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "cold:function:method\ncold:function:method"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_writes_back_direct_array_offset_reference_arguments() {
    let execution = run_source(
        r#"<?php
function mark_slot(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$_REQUEST["payload"] = ["slot" => "request"];
$GLOBALS["bag"] = ["slot" => "global"];
$items = ["outer" => ["slot" => "array"]];

echo call_user_func_array("mark_slot", array(&$_REQUEST["payload"]["slot"], "request-callback")), "\n";
call_user_func_array("mark_slot", array(&$GLOBALS["bag"]["slot"], "global-callback"));
call_user_func_array("mark_slot", array(&$items["outer"]["slot"], "array-callback"));
echo $_REQUEST["payload"]["slot"], "|", $GLOBALS["bag"]["slot"], "|", $items["outer"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request:request-callback\nrequest:request-callback|global:global-callback|array:array-callback"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_reference_return_binds_direct_array_offset_arguments() {
    let execution = run_source(
        r#"<?php
function &tag_slot(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$_REQUEST["payload"] = ["slot" => "request"];
$GLOBALS["bag"] = ["slot" => "global"];
$items = ["outer" => ["slot" => "array"]];

$request_alias =& call_user_func_array("tag_slot", array(&$_REQUEST["payload"]["slot"], "request"));
$request_alias = $request_alias . ":alias";

$global_alias =& call_user_func_array("tag_slot", array(&$GLOBALS["bag"]["slot"], "global"));
$global_alias = $global_alias . ":alias";

$array_alias =& call_user_func_array("tag_slot", array(&$items["outer"]["slot"], "array"));
$array_alias = $array_alias . ":alias";

echo $_REQUEST["payload"]["slot"], "|", $request_alias, "\n";
echo $GLOBALS["bag"]["slot"], "|", $global_alias, "\n";
echo $items["outer"]["slot"], "|", $array_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request:request:alias|request:request:alias\nglobal:global:alias|global:global:alias\narray:array:alias|array:array:alias"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_writes_back_static_array_callable_reference_arguments() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$option = "start";
echo call_user_func_array(array("Cache_Marker", "tag"), array(&$option, "direct")), "\n";
call_user_func_array(array("Cache_Marker", "tag"), array(&$cache->cache["options"]["alloptions"], "static"));
echo $option, "|", $cache->cache["options"]["alloptions"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "start:direct\nstart:direct|cold:static");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_accepts_integer_keyed_reference_argument_arrays() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];
}

class OptionFilter {
    public function update(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function update_option_like(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
echo call_user_func_array("update_option_like", array(2 => &$option, 7 => "cache")), "\n";
$filter = new OptionFilter();
echo call_user_func_array(array($filter, "update"), array(10 => &$option, 20 => "object")), "\n";
$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
call_user_func_array(array("Cache_Marker", "tag"), array(4 => &$cache->cache["options"]["alloptions"], 6 => "static"));
echo $option, "|", $cache->cache["options"]["alloptions"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:cache\nautoload:cache:object\nautoload:cache:object|cold:static"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_stored_reference_argument_arrays() {
    let execution = run_source(
        r#"<?php
class OptionFilter {
    public function update(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class WP_Object_Cache {
    public $cache = [];
}

function update_option_like(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
$args = [];
$args[10] =& $option;
$args[20] = "stored";
echo call_user_func_array("update_option_like", $args), "\n";

$copy = $args;
$copy[20] = "copy";
$filter = new OptionFilter();
echo call_user_func_array(array($filter, "update"), $copy), "\n";

$_REQUEST["mode"] = "draft";
$request_alias =& $_REQUEST["mode"];
$request_args = [];
$request_args[0] =& $request_alias;
$request_args[1] = "request";
call_user_func_array("update_option_like", $request_args);
echo $_REQUEST["mode"], "|", $request_args[0], "\n";

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$cache_slot =& $cache->cache["options"]["alloptions"];
$static_args = [];
$static_args[0] =& $cache_slot;
$static_args[1] = "static";
call_user_func_array(array("Cache_Marker", "tag"), $static_args);
echo $option, "|", $cache->cache["options"]["alloptions"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:stored\nautoload:stored:copy\ndraft:request|draft:request\nautoload:stored:copy|cold:static"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_allows_reference_returning_callbacks_with_array_offset_bindings() {
    let execution = run_source(
        r#"<?php
class OptionFilter {
    public function &mark(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class Cache_Marker {
    public static function &tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class WP_Object_Cache {
    public $cache = [];
}

function &tag_option(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
$args = [];
$args[0] =& $option;
$args[1] = "stored";
echo call_user_func_array("tag_option", $args), "|", $option, "|", $args[0], "\n";

$copy = $args;
$copy[1] = "method";
$filter = new OptionFilter();
echo call_user_func_array(array($filter, "mark"), $copy), "|", $option, "\n";

$_REQUEST["mode"] = "draft";
$request_alias =& $_REQUEST["mode"];
$request_args = [];
$request_args[0] =& $request_alias;
$request_args[1] = "request";
echo call_user_func_array("tag_option", $request_args), "|", $_REQUEST["mode"], "|", $request_args[0], "\n";

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$cache_slot =& $cache->cache["options"]["alloptions"];
$static_args = [];
$static_args[0] =& $cache_slot;
$static_args[1] = "static";
echo call_user_func_array(array("Cache_Marker", "tag"), $static_args), "|", $cache->cache["options"]["alloptions"], "\n";

$cache->cache["options"]["runtime"] = "warm";
echo call_user_func_array("tag_option", array(&$cache->cache["options"]["runtime"], "literal")), "|", $cache->cache["options"]["runtime"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:stored|autoload:stored|autoload:stored\nautoload:stored:method|autoload:stored:method\ndraft:request|draft:request|draft:request\ncold:static|cold:static\nwarm:literal|warm:literal"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_reference_return_sources_to_direct_variable_arguments() {
    let execution = run_source(
        r#"<?php
class OptionFilter {
    public function &mark(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class Cache_Marker {
    public static function &tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function &tag_option(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
$alias =& call_user_func_array("tag_option", array(&$option, "function"));
$alias = $alias . ":alias";
echo $option, "|", $alias, "\n";

$filter = new OptionFilter();
$method_alias =& call_user_func_array(array($filter, "mark"), array(&$option, "method"));
$option = "root";
echo $method_alias, "|", $alias, "\n";

$static_alias =& call_user_func_array(array("Cache_Marker", "tag"), array(&$option, "static"));
$static_alias = $static_alias . ":done";
echo $option, "|", $method_alias, "|", $static_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:function:alias|autoload:function:alias\nroot|root\nroot:static:done|root:static:done|root:static:done"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_reference_return_sources_to_object_property_array_arguments() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];
}

class OptionFilter {
    public function &mark(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class Cache_Marker {
    public static function &tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function &wp_refcow_tag(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$alias =& call_user_func_array("wp_refcow_tag", array(&$cache->cache["options"]["alloptions"], "function"));
$alias = $alias . ":alias";
echo $cache->cache["options"]["alloptions"], "|", $alias, "\n";

$filter = new OptionFilter();
$method_alias =& call_user_func_array(array($filter, "mark"), array(&$cache->cache["options"]["alloptions"], "method"));
$cache->cache["options"]["alloptions"] = "root";
echo $method_alias, "|", $alias, "\n";

$static_alias =& call_user_func_array(array("Cache_Marker", "tag"), array(&$cache->cache["options"]["alloptions"], "static"));
$static_alias = $static_alias . ":done";
echo $cache->cache["options"]["alloptions"], "|", $method_alias, "|", $static_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "cold:function:alias|cold:function:alias\nroot|root\nroot:static:done|root:static:done|root:static:done"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_reference_return_sources_to_stored_reference_argument_arrays() {
    let execution = run_source(
        r#"<?php
class WP_Object_Cache {
    public $cache = [];
}

class OptionFilter {
    public function &mark(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class Cache_Marker {
    public static function &tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function &tag_option(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
$args = [];
$args[0] =& $option;
$args[1] = "function";
$alias =& call_user_func_array("tag_option", $args);
$alias = $alias . ":alias";
echo $option, "|", $args[0], "|", $alias, "\n";

$_REQUEST["mode"] = "draft";
$request_alias =& $_REQUEST["mode"];
$request_args = [];
$request_args[0] =& $request_alias;
$request_args[1] = "request";
$request_result =& call_user_func_array("tag_option", $request_args);
$request_result = $request_result . ":seen";
echo $_REQUEST["mode"], "|", $request_args[0], "|", $request_result, "\n";

$filter = new OptionFilter();
$method_alias =& call_user_func_array(array($filter, "mark"), $args);
$option = "root";
echo $method_alias, "|", $alias, "\n";

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$cache_slot =& $cache->cache["options"]["alloptions"];
$static_args = [];
$static_args[0] =& $cache_slot;
$static_args[1] = "static";
$static_alias =& call_user_func_array(array("Cache_Marker", "tag"), $static_args);
$static_alias = $static_alias . ":done";
echo $cache->cache["options"]["alloptions"], "|", $static_args[0], "|", $static_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:function:alias|autoload:function:alias|autoload:function:alias\ndraft:request:seen|draft:request:seen|draft:request:seen\nroot|root\ncold:static:done|cold:static:done|cold:static:done"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_alias_backed_literal_reference_arguments() {
    let execution = run_source(
        r#"<?php
function wp_refcow_mark_payload(&$payload, $key, $suffix) {
    $payload[$key] = $payload[$key] . ":" . $suffix;
    return $payload[$key];
}

function &wp_refcow_pick_payload(&$payload, $key, $suffix) {
    $payload[$key] = $payload[$key] . ":" . $suffix;
    return $payload[$key];
}

class WP_RefCow_Callback {
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
call_user_func_array("wp_refcow_mark_payload", array(&$request_payload, "slot", "normal"));
echo $_REQUEST["payload"]["slot"], "|", $request_payload["slot"], "\n";

$items = ["outer" => ["slot" => "array"]];
$outer =& $items["outer"];
$function_alias =& call_user_func_array("wp_refcow_pick_payload", array(&$outer, "slot", "function"));
$function_alias = $function_alias . ":alias";
echo $items["outer"]["slot"], "|", $outer["slot"], "|", $function_alias, "\n";

$method_items = ["outer" => ["slot" => "method"]];
$method_parent =& $method_items["outer"];
$callback = new WP_RefCow_Callback();
$method_alias =& call_user_func_array(array($callback, "pick"), array(&$method_parent, "slot", "method"));
$method_items["outer"]["slot"] = $method_items["outer"]["slot"] . ":root";
echo $method_items["outer"]["slot"], "|", $method_parent["slot"], "|", $method_alias, "\n";

$static_items = ["outer" => ["slot" => "static"]];
$static_parent =& $static_items["outer"];
$static_alias =& call_user_func_array(array("WP_RefCow_Callback", "pickStatic"), array(&$static_parent, "slot", "static"));
$static_alias = $static_alias . ":alias";
echo $static_items["outer"]["slot"], "|", $static_parent["slot"], "|", $static_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "request:normal|request:normal\narray:function:alias|array:function:alias|array:function:alias\nmethod:method:root|method:method:root|method:method:root\nstatic:static:alias|static:static:alias|static:static:alias"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_invokes_current_array_callable_subset() {
    let execution = run_source(
        r#"<?php
class Formatter {
    public $prefix = ">";

    public function wrap($value) {
        return $this->prefix . $value;
    }

    public static function join($left, $right) {
        return $left . ":" . $right;
    }
}

$formatter = new Formatter();
echo call_user_func_array(array($formatter, "wrap"), array("item")), "\n";
echo call_user_func_array(array("Formatter", "join"), array("a", "b"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, ">item\na:b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_alias_backed_stored_reference_argument_arrays() {
    let execution = run_source(
        r#"<?php
class WP_RefCow_Stored_Callback {
    public $store = [];

    public function &pick(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }

    public static function &pickStatic(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function wp_refcow_stored_mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &wp_refcow_stored_pick(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
$registry = [];
$registry["args"] = [];
$args =& $registry["args"];
$args[0] =& $option;
$args[1] = "normal";
call_user_func_array("wp_refcow_stored_mark", $args);
echo $option, "|", $args[0], "|", $registry["args"][0], "\n";

$alias =& call_user_func_array("wp_refcow_stored_pick", $args);
$alias = $alias . ":alias";
echo $option, "|", $args[0], "|", $alias, "\n";

$_REQUEST["callback_args"] = [];
$request_args =& $_REQUEST["callback_args"];
$request_mode =& $_REQUEST["mode"];
$request_mode = "draft";
$request_args[0] =& $request_mode;
$request_args[1] = "request";
$callback = new WP_RefCow_Stored_Callback();
$method_alias =& call_user_func_array(array($callback, "pick"), $request_args);
$method_alias = $method_alias . ":method";
echo $_REQUEST["mode"], "|", $_REQUEST["callback_args"][0], "|", $method_alias, "\n";

$box = new WP_RefCow_Stored_Callback();
$box->store["args"] = [];
$object_args =& $box->store["args"];
$cache = "cold";
$object_args[0] =& $cache;
$object_args[1] = "static";
$static_alias =& call_user_func_array(array("WP_RefCow_Stored_Callback", "pickStatic"), $object_args);
$static_alias = $static_alias . ":done";
echo $cache, "|", $box->store["args"][0], "|", $static_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:normal|autoload:normal|autoload:normal\nautoload:normal:normal:alias|autoload:normal:normal:alias|autoload:normal:normal:alias\ndraft:request:method|draft:request:method|draft:request:method\ncold:static:done|cold:static:done|cold:static:done"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_stored_append_reference_argument_roots() {
    let execution = run_source(
        r#"<?php
function wp_refcow_append_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_append_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

class WP_RefCow_Append_Store {
    public $items = [];
}

$items = [];
$args = [];
$args[] =& $items[];
$args[] = "direct";
echo call_user_func_array("wp_refcow_append_mark", $args), "|", $items[0], "|", $args[0], "\n";

$alias =& call_user_func_array("wp_refcow_append_pick", $args);
$alias = $alias . ":alias";
echo $items[0], "|", $args[0], "|", $alias, "\n";

$store = new WP_RefCow_Append_Store();
$named = [];
$named["value"] =& $store->items[];
$named["suffix"] = "property";
echo call_user_func_array("wp_refcow_append_mark", $named), "|", $store->items[0], "|", $named["value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "null:direct|null:direct|null:direct\nnull:direct:direct:alias|null:direct:direct:alias|null:direct:direct:alias\nnull:property|null:property|null:property"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_visible_non_public_object_property_array_reference_arguments() {
    let execution = run_source(
        r#"<?php
function wp_refcow_mark_non_public(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &wp_refcow_pick_non_public(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

class WP_RefCow_Non_Public_Store {
    private $privateStore = ["slot" => "private", "direct" => "direct"];
    protected $protectedStore = ["slot" => "protected"];

    public function probe($peer) {
        wp_refcow_mark_non_public($this->privateStore["direct"], "call");

        call_user_func_array("wp_refcow_mark_non_public", array(&$this->privateStore["slot"], "mark"));
        $alias =& call_user_func_array("wp_refcow_pick_non_public", array(&$this->privateStore["slot"], "pick"));
        $alias = $alias . ":alias";

        call_user_func_array("wp_refcow_mark_non_public", array(&$peer->protectedStore["slot"], "peer"));

        echo $this->privateStore["direct"], "|", $this->privateStore["slot"], "|", $alias, "|", $peer->protectedStore["slot"];
    }
}

$left = new WP_RefCow_Non_Public_Store();
$right = new WP_RefCow_Non_Public_Store();
$left->probe($right);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct:call|private:mark:pick:alias|private:mark:pick:alias|protected:peer"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_non_public_stored_object_property_reference_argument_arrays() {
    let execution = run_source(
        r#"<?php
function wp_refcow_mark_stored_non_public(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &wp_refcow_pick_stored_non_public(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

class WP_RefCow_Stored_Non_Public_Store {
    private $privateArgs = [];
    private $privateStore = ["slot" => "private"];
    protected $protectedArgs = [];
    protected $protectedStore = ["slot" => "protected"];

    public function probe($peer) {
        $privateSlot =& $this->privateStore["slot"];
        $this->privateArgs[0] =& $privateSlot;
        $this->privateArgs[1] = "mark";
        call_user_func_array("wp_refcow_mark_stored_non_public", $this->privateArgs);

        $this->privateArgs[1] = "pick";
        $alias =& call_user_func_array("wp_refcow_pick_stored_non_public", $this->privateArgs);
        $alias = $alias . ":alias";

        $protectedSlot =& $peer->protectedStore["slot"];
        $peer->protectedArgs[0] =& $protectedSlot;
        $peer->protectedArgs[1] = "peer";
        call_user_func_array("wp_refcow_mark_stored_non_public", $peer->protectedArgs);

        echo $this->privateStore["slot"], "|", $this->privateArgs[0], "|", $alias, "|", $peer->protectedStore["slot"], "|", $peer->protectedArgs[0];
    }
}

$left = new WP_RefCow_Stored_Non_Public_Store();
$right = new WP_RefCow_Stored_Non_Public_Store();
$left->probe($right);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "private:mark:pick:alias|private:mark:pick:alias|private:mark:pick:alias|protected:peer|protected:peer"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_reports_nested_positional_after_named_before_callback_lookup() {
    let execution = run_source(
        r#"<?php
call_user_func_array("missing_func", array_slice(array: $args, 1, 2));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Fatal error: Cannot use positional argument after named argument in Command line code on line 2"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn call_user_func_array_binds_dynamic_string_keyed_reference_argument_arrays() {
    let execution = run_source(
        r#"<?php
function wp_refcow_dynamic_named(&$first, &$second, $label = "ok") {
    $first = $first . ":first";
    $second = $second . ":second";
    echo $label, ":", $first, ":", $second, "
";
}

$left = "L";
$right = "R";
$secondKey = "second";
$firstKey = "fir" . "st";
call_user_func_array("wp_refcow_dynamic_named", array($secondKey => &$right, $firstKey => &$left));
echo $left, "|", $right;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ok:L:first:R:second
L:first|R:second"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_string_keyed_reference_argument_arrays() {
    let execution = run_source(
        r#"<?php
function wp_refcow_named_mark(&$value, $suffix = "default") {
    $value = $value . ":" . $suffix;
    return $value;
}

function &wp_refcow_named_pick(&$value, $suffix = "default") {
    $value = $value . ":" . $suffix;
    return $value;
}

class WP_RefCow_Named_Callback {
    public function &pick(&$value, $suffix = "default") {
        $value = $value . ":" . $suffix;
        return $value;
    }

    public static function &pickStatic(&$value, $suffix = "default") {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

$option = "autoload";
echo call_user_func_array("wp_refcow_named_mark", array("suffix" => "literal", "value" => &$option)), "|", $option, "\n";

$alias =& call_user_func_array("wp_refcow_named_pick", array("suffix" => "return", "value" => &$option));
$alias = $alias . ":alias";
echo $option, "|", $alias, "\n";

$_REQUEST["mode"] = "draft";
$request_alias =& $_REQUEST["mode"];
$stored = [];
$stored["value"] =& $request_alias;
$stored["suffix"] = "stored";
echo call_user_func_array("wp_refcow_named_mark", $stored), "|", $_REQUEST["mode"], "|", $stored["value"], "\n";

$callback = new WP_RefCow_Named_Callback();
$method_alias =& call_user_func_array(array($callback, "pick"), array("suffix" => "method", "value" => &$request_alias));
$method_alias = $method_alias . ":method-alias";
echo $_REQUEST["mode"], "|", $method_alias, "\n";

$static_alias =& call_user_func_array(array("WP_RefCow_Named_Callback", "pickStatic"), $stored);
$static_alias = $static_alias . ":static-alias";
echo $_REQUEST["mode"], "|", $stored["value"], "|", $static_alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:literal|autoload:literal\nautoload:literal:return:alias|autoload:literal:return:alias\ndraft:stored|draft:stored|draft:stored\ndraft:stored:method:method-alias|draft:stored:method:method-alias\ndraft:stored:method:method-alias:stored:static-alias|draft:stored:method:method-alias:stored:static-alias|draft:stored:method:method-alias:stored:static-alias"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_magic_get_array_reference_arguments() {
    let execution = run_source(
        r#"<?php
$storage = array("slot" => "initial", "return" => array("leaf" => "seed"), "named" => "name");

class WP_RefCow_Magic_Get_Callback_Box {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

function wp_refcow_magic_callback_mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

function &wp_refcow_magic_callback_pick(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$box = new WP_RefCow_Magic_Get_Callback_Box();
echo call_user_func_array("wp_refcow_magic_callback_mark", array(&$box->missing["slot"], "normal")), "|", $storage["slot"], "\n";

$alias =& call_user_func_array("wp_refcow_magic_callback_pick", array(&$box->missing["return"]["leaf"], "return"));
$alias = $alias . ":alias";
echo $storage["return"]["leaf"], "|", $alias, "\n";

echo call_user_func_array("wp_refcow_magic_callback_mark", array("suffix" => "named", "value" => &$box->missing["named"])), "|", $storage["named"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "initial:normal|initial:normal\nseed:return:alias|seed:return:alias\nname:named|name:named"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_stored_magic_get_array_reference_arguments() {
    let execution = run_source(
        r#"<?php
$storage = array("slot" => "initial", "return" => array("leaf" => "seed"), "named" => "name");

class WP_RefCow_Stored_Magic_Get_Callback_Box {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

function wp_refcow_stored_magic_callback_mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

function &wp_refcow_stored_magic_callback_pick(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$box = new WP_RefCow_Stored_Magic_Get_Callback_Box();
$args = array(&$box->missing["slot"], "normal");
echo call_user_func_array("wp_refcow_stored_magic_callback_mark", $args), "|", $storage["slot"], "|", $args[0], "\n";

$return_args = array(&$box->missing["return"]["leaf"], "return");
$alias =& call_user_func_array("wp_refcow_stored_magic_callback_pick", $return_args);
$alias = $alias . ":alias";
echo $storage["return"]["leaf"], "|", $return_args[0], "|", $alias, "\n";

$named = array("suffix" => "named", "value" => &$box->missing["named"]);
echo call_user_func_array("wp_refcow_stored_magic_callback_mark", $named), "|", $storage["named"], "|", $named["value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "initial:normal|initial:normal|initial:normal\nseed:return:alias|seed:return:alias|seed:return:alias\nname:named|name:named|name:named"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_stored_magic_get_nested_array_access_references() {
    let execution = run_source(
        r#"<?php
class WP_RefCow_Stored_Magic_Nested_ArrayAccess_Bag implements ArrayAccess {
    private $storage;

    public function __construct($storage = []) {
        $this->storage = $storage;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->storage[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->storage[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->storage[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->storage[$offset]);
    }

    public function read($offset) {
        return $this->storage[$offset];
    }
}

$inner = new WP_RefCow_Stored_Magic_Nested_ArrayAccess_Bag(["slot" => "seed", "return" => "pick"]);
$outer = new WP_RefCow_Stored_Magic_Nested_ArrayAccess_Bag(["inner" => $inner]);

class WP_RefCow_Stored_Magic_Nested_ArrayAccess_Box {
    public function &__get($name) {
        global $outer;
        return $outer;
    }
}

function wp_refcow_stored_magic_nested_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_stored_magic_nested_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$box = new WP_RefCow_Stored_Magic_Nested_ArrayAccess_Box();
$args = array(&$box->missing["inner"]["slot"], "stored");
echo call_user_func_array("wp_refcow_stored_magic_nested_array_access_mark", $args), "|", $inner->read("slot"), "|", $args[0], "\n";

$returnArgs = array(&$box->missing["inner"]["return"], "return");
$alias =& call_user_func_array("wp_refcow_stored_magic_nested_array_access_pick", $returnArgs);
$alias = $alias . ":alias";
echo $inner->read("return"), "|", $returnArgs[0], "|", $alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:stored|seed:stored|seed:stored\npick:return:alias|pick:return:alias|pick:return:alias"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_array_access_reference_roots() {
    let execution = run_source(
        r#"<?php
class WP_RefCow_ArrayAccess_Bag implements ArrayAccess {
    private $items = ["slot" => "seed", "outer" => ["slot" => "nested"]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

function wp_refcow_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

class WP_RefCow_ArrayAccess_Holder {
    public $bag;
}

$bag = new WP_RefCow_ArrayAccess_Bag();
$alias =& $bag["slot"];
$alias = $alias . ":alias";
echo $bag["slot"], "|", $alias, "\n";

echo call_user_func_array("wp_refcow_array_access_mark", array(&$bag["slot"], "callback")), "|", $bag["slot"], "\n";

$picked =& call_user_func_array("wp_refcow_array_access_pick", array(&$bag["missing"], "return"));
$picked = $picked . ":picked";
echo $bag["missing"], "|", $picked, "\n";

echo call_user_func_array("wp_refcow_array_access_mark", array(&$bag["outer"]["slot"], "nested-callback")), "|", $bag["outer"]["slot"], "\n";

$nested =& call_user_func_array("wp_refcow_array_access_pick", array(&$bag["created"]["leaf"], "nested-return"));
$nested = $nested . ":nested-alias";
echo $bag["created"]["leaf"], "|", $nested, "\n";

$holder = new WP_RefCow_ArrayAccess_Holder();
$holder->bag = new WP_RefCow_ArrayAccess_Bag();
echo call_user_func_array("wp_refcow_array_access_mark", array(&$holder->bag["outer"]["slot"], "held-callback")), "|", $holder->bag["outer"]["slot"], "\n";
$held =& call_user_func_array("wp_refcow_array_access_pick", array(&$holder->bag["held"]["leaf"], "held-return"));
$held = $held . ":held-alias";
echo $holder->bag["held"]["leaf"], "|", $held, "\n";

$stored = [];
$stored[0] =& $bag["stored"];
$stored[1] = "stored-callback";
call_user_func_array("wp_refcow_array_access_mark", $stored);
echo $bag["stored"], "|", $stored[0], "\n";

$heldStored = [];
$heldStored[] =& $holder->bag["heldStored"]["leaf"];
$heldStored[] = "held-stored";
call_user_func_array("wp_refcow_array_access_mark", $heldStored);
echo $holder->bag["heldStored"]["leaf"], "|", $heldStored[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:alias|seed:alias\nseed:alias:callback|seed:alias:callback\nnull:return:picked|null:return:picked\nnested:nested-callback|nested:nested-callback\nnull:nested-return:nested-alias|null:nested-return:nested-alias\nnested:held-callback|nested:held-callback\nnull:held-return:held-alias|null:held-return:held-alias\nnull:stored-callback|null:stored-callback\nnull:held-stored|null:held-stored"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_append_array_access_reference_roots() {
    let execution = run_source(
        r#"<?php
class WP_RefCow_Append_ArrayAccess_Bag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class WP_RefCow_Append_ArrayAccess_Holder {
    public $bag;
}

function wp_refcow_append_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_append_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$bag = new WP_RefCow_Append_ArrayAccess_Bag();
$direct =& $bag[];
$direct = "direct";
echo $bag->items[""], "|", $direct, "\n";

$args = [];
$args[0] =& $bag[];
$args[1] = "stored";
call_user_func_array("wp_refcow_append_array_access_mark", $args);
echo $bag->items[""], "|", $args[0], "\n";

$alias =& call_user_func_array("wp_refcow_append_array_access_pick", $args);
$alias = $alias . ":alias";
echo $bag->items[""], "|", $args[0], "|", $alias, "\n";

$holder = new WP_RefCow_Append_ArrayAccess_Holder();
$holder->bag = new WP_RefCow_Append_ArrayAccess_Bag();
$held = [];
$held["value"] =& $holder->bag[];
$held["suffix"] = "held";
call_user_func_array("wp_refcow_append_array_access_mark", $held);
echo $holder->bag->items[""], "|", $held["value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct|direct\ndirect:stored|direct:stored\ndirect:stored:stored:alias|direct:stored:stored:alias|direct:stored:stored:alias\nnull:held|null:held"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_dynamic_property_array_access_reference_roots() {
    let execution = run_source(
        r#"<?php
class WP_RefCow_Dynamic_Property_ArrayAccess_Bag implements ArrayAccess {
    private $items = ["slot" => "seed", "outer" => ["slot" => "nested"]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class WP_RefCow_Dynamic_Property_ArrayAccess_Holder {
    public $bag;
}

function wp_refcow_dynamic_property_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_dynamic_property_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$name = "bag";
$holder = new WP_RefCow_Dynamic_Property_ArrayAccess_Holder();
$holder->bag = new WP_RefCow_Dynamic_Property_ArrayAccess_Bag();

$alias =& $holder->{$name}["slot"];
$alias = $alias . ":alias";
echo $holder->bag["slot"], "|", $alias, "\n";

echo call_user_func_array("wp_refcow_dynamic_property_array_access_mark", array(&$holder->{$name}["outer"]["slot"], "callback")), "|", $holder->bag["outer"]["slot"], "\n";

$stored = [];
$stored["value"] =& $holder->{$name}["created"]["leaf"];
$stored["suffix"] = "stored";
call_user_func_array("wp_refcow_dynamic_property_array_access_mark", $stored);
echo $holder->bag["created"]["leaf"], "|", $stored["value"], "\n";

$picked =& call_user_func_array("wp_refcow_dynamic_property_array_access_pick", array(&$holder->{$name}["return"], "pick"));
$picked = $picked . ":picked";
echo $holder->bag["return"], "|", $picked;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:alias|seed:alias\nnested:callback|nested:callback\nnull:stored|null:stored\nnull:pick:picked|null:pick:picked"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_context_dynamic_property_array_access_reference_roots() {
    let execution = run_source(
        r#"<?php
class WP_RefCow_Context_Dynamic_Property_ArrayAccess_Bag implements ArrayAccess {
    private $items = ["slot" => "seed", "outer" => ["slot" => "nested"]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class WP_RefCow_Context_Dynamic_Property_ArrayAccess_Holder_Base {
    protected $protectedBag;
}

class WP_RefCow_Context_Dynamic_Property_ArrayAccess_Holder extends WP_RefCow_Context_Dynamic_Property_ArrayAccess_Holder_Base {
    private $privateBag;

    public function exercise() {
        $private = "privateBag";
        $protected = "protectedBag";
        $this->privateBag = new WP_RefCow_Context_Dynamic_Property_ArrayAccess_Bag();
        $this->protectedBag = new WP_RefCow_Context_Dynamic_Property_ArrayAccess_Bag();

        $alias =& $this->{$private}["slot"];
        $alias = $alias . ":alias";
        echo $this->{$private}["slot"], "|", $alias, "\n";

        echo call_user_func_array("wp_refcow_context_dynamic_property_array_access_mark", array(&$this->{$private}["outer"]["slot"], "callback")), "|", $this->{$private}["outer"]["slot"], "\n";

        $stored = [];
        $stored["value"] =& $this->{$protected}["created"]["leaf"];
        $stored["suffix"] = "stored";
        call_user_func_array("wp_refcow_context_dynamic_property_array_access_mark", $stored);
        echo $this->{$protected}["created"]["leaf"], "|", $stored["value"], "\n";

        $picked =& call_user_func_array("wp_refcow_context_dynamic_property_array_access_pick", array(&$this->{$protected}["return"], "pick"));
        $picked = $picked . ":picked";
        echo $this->{$protected}["return"], "|", $picked;
    }
}

function wp_refcow_context_dynamic_property_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_context_dynamic_property_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$holder = new WP_RefCow_Context_Dynamic_Property_ArrayAccess_Holder();
$holder->exercise();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:alias|seed:alias\nnested:callback|nested:callback\nnull:stored|null:stored\nnull:pick:picked|null:pick:picked"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_maps_string_keyed_value_argument_arrays() {
    let execution = run_source(
        r#"<?php
function wp_refcow_named_value($value, $suffix = "default", $prefix = "pre") {
    return $prefix . ":" . $value . ":" . $suffix;
}

class WP_RefCow_Named_Value_Callback {
    public function wrap($value, $suffix = "method", $prefix = "box") {
        return $prefix . ":" . $value . ":" . $suffix;
    }

    public static function join($value, $suffix = "static", $prefix = "class") {
        return $prefix . ":" . $value . ":" . $suffix;
    }
}

echo call_user_func_array("wp_refcow_named_value", array("suffix" => "literal", "value" => "cache")), "\n";

$stored = array("prefix" => "stored", "value" => "option");
echo call_user_func_array("wp_refcow_named_value", $stored), "\n";

$callback = new WP_RefCow_Named_Value_Callback();
echo call_user_func_array(array($callback, "wrap"), array("suffix" => "object", "value" => "payload")), "\n";
echo call_user_func_array(array("WP_RefCow_Named_Value_Callback", "join"), array("prefix" => "static", "value" => "payload"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "pre:cache:literal\nstored:option:default\nbox:payload:object\nstatic:payload:static"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_is_available_through_function_metadata_builtins() {
    let execution = run_source(
        r#"<?php
echo function_exists("call_user_func") ? "yes" : "no";
echo "|";
echo is_callable("call_user_func") ? "callable" : "missing";
echo "|";
echo function_exists("call_user_func_array") ? "yes" : "no";
echo "|";
echo is_callable("call_user_func_array") ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|yes|callable");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn iterator_current_bucket_copy_preserves_nested_reference_slots() {
    let execution = run_source(
        r#"<?php
class WP_RefCow_Hook_Iterator implements Iterator {
    public $callbacks = array();
    public $priorities = array(10);
    public $index = 0;

    public function rewind() {
        $this->index = 0;
    }

    public function valid() {
        return $this->index < count($this->priorities);
    }

    public function key() {
        return $this->priorities[$this->index];
    }

    public function next() {
        $this->index = $this->index + 1;
    }

    public function current() {
        return $this->callbacks[$this->priorities[$this->index]];
    }
}

class WP_RefCow_Hook_Ref_Iterator extends WP_RefCow_Hook_Iterator {
    public function &current() {
        return $this->callbacks[$this->priorities[$this->index]];
    }
}

$target = "seed";
$hook = new WP_RefCow_Hook_Iterator();
$hook->callbacks[10] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
);
$hook->callbacks[10]["id"]["function"] =& $target;
foreach ($hook as $priority => $bucket) {
    $bucket["id"]["accepted_args"] = 99;
    foreach ($bucket as $id => &$callback) {
        $callback["function"] = $callback["function"] . ":value";
    }
}
echo $target, "|", $hook->callbacks[10]["id"]["function"], "|", $hook->callbacks[10]["id"]["accepted_args"], "\n";

$refTarget = "seed";
$refHook = new WP_RefCow_Hook_Ref_Iterator();
$refHook->callbacks[10] = array(
    "id" => array("function" => "placeholder", "accepted_args" => 1),
);
$refHook->callbacks[10]["id"]["function"] =& $refTarget;
foreach ($refHook as $priority => $bucket) {
    $bucket["id"]["accepted_args"] = 99;
    foreach ($bucket as $id => &$callback) {
        $callback["function"] = $callback["function"] . ":ref";
    }
}
echo $refTarget, "|", $refHook->callbacks[10]["id"]["function"], "|", $refHook->callbacks[10]["id"]["accepted_args"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:value|seed:value|1\nseed:ref|seed:ref|1"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_non_array_arguments_throw_type_error() {
    let execution = run_source(
        r#"<?php
function cufa_typeerror_target($value) {
    return $value;
}

class CufaTypeErrorTarget {
    public static function tag($value) {
        return $value;
    }
}

$closure = function ($value) {
    return $value;
};
$notArray = "four";
$callbacks = array(
    "strlen",
    "cufa_typeerror_target",
    array("CufaTypeErrorTarget", "tag"),
    $closure,
);

foreach ($callbacks as $callback) {
    try {
        call_user_func_array($callback, $notArray);
    } catch (TypeError $e) {
        echo get_class($e), "|", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "TypeError|call_user_func_array(): Argument #2 ($args) must be of type array, string given\n",
            "TypeError|call_user_func_array(): Argument #2 ($args) must be of type array, string given\n",
            "TypeError|call_user_func_array(): Argument #2 ($args) must be of type array, string given\n",
            "TypeError|call_user_func_array(): Argument #2 ($args) must be of type array, string given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_rejects_forms_outside_current_subset() {
    let missing = runtime_error(
        r#"<?php
echo call_user_func();
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for call_user_func(): expected at least 1 argument(s), got 0"
    );

    let non_string = runtime_error(
        r#"<?php
echo call_user_func(42);
"#,
    );
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 6);
    assert_eq!(
        non_string.message,
        "unsupported call call_user_func(): callback must evaluate to string in the current subset, got int"
    );

    let malformed_array_callable = runtime_error(
        r#"<?php
echo call_user_func(["ClassName"]);
"#,
    );
    assert_eq!(malformed_array_callable.line, 2);
    assert_eq!(malformed_array_callable.column, 6);
    assert_eq!(
        malformed_array_callable.message,
        "unsupported call call_user_func(): array callback must be [object-or-class, method] in the current subset"
    );

    let unknown = runtime_error(
        r#"<?php
echo call_user_func("missing_function");
"#,
    );
    assert_eq!(unknown.line, 2);
    assert_eq!(unknown.column, 6);
    assert_eq!(unknown.message, "undefined function missing_function()");

    let missing_array_arg = runtime_error(
        r#"<?php
echo call_user_func_array("strlen");
"#,
    );
    assert_eq!(missing_array_arg.line, 2);
    assert_eq!(missing_array_arg.column, 6);
    assert_eq!(
        missing_array_arg.message,
        "arity mismatch for call_user_func_array(): expected 2 argument(s), got 1"
    );

    let named_args = runtime_error(
        r#"<?php
echo call_user_func_array("strlen", array("value" => "four"));
"#,
    );
    assert_eq!(named_args.line, 2);
    assert_eq!(named_args.column, 6);
    assert_eq!(
        named_args.message,
        "unsupported call call_user_func_array(): string-keyed named arguments are not implemented in the current subset"
    );

    let unknown_named_reference_args = runtime_error(
        r#"<?php
function mutate(&$value) {
    $value = "changed";
}
$option = "original";
call_user_func_array("mutate", array("missing" => &$option));
"#,
    );
    assert_eq!(unknown_named_reference_args.line, 6);
    assert_eq!(unknown_named_reference_args.column, 1);
    assert_eq!(
        unknown_named_reference_args.message,
        "unsupported call mutate(): call_user_func_array() named argument $missing does not match a declared parameter in the current subset"
    );

    let duplicate_named_value_args = runtime_error(
        r#"<?php
function format_option($value, $suffix = "default") {
    return $value . ":" . $suffix;
}
call_user_func_array("format_option", array("cache", "value" => "override"));
"#,
    );
    assert_eq!(duplicate_named_value_args.line, 5);
    assert_eq!(duplicate_named_value_args.column, 1);
    assert_eq!(
        duplicate_named_value_args.message,
        "unsupported call format_option(): call_user_func_array() duplicate argument for parameter $value is not implemented in the current subset"
    );

    let unknown_named_value_args = runtime_error(
        r#"<?php
function format_option($value, $suffix = "default") {
    return $value . ":" . $suffix;
}
call_user_func_array("format_option", array("missing" => "override"));
"#,
    );
    assert_eq!(unknown_named_value_args.line, 5);
    assert_eq!(unknown_named_value_args.column, 1);
    assert_eq!(
        unknown_named_value_args.message,
        "unsupported call format_option(): call_user_func_array() named argument $missing does not match a declared parameter in the current subset"
    );

    let stored_by_value_args = runtime_error(
        r#"<?php
function mutate(&$value) {
    $value = "changed";
}
$option = "original";
$args = [];
$args[0] = $option;
call_user_func_array("mutate", $args);
"#,
    );
    assert_eq!(stored_by_value_args.line, 8);
    assert_eq!(stored_by_value_args.column, 32);
    assert_eq!(
        stored_by_value_args.message,
        "unsupported call mutate(): call_user_func_array() stored reference parameter invocation requires each reached by-reference argument slot to have been assigned by reference in the current subset"
    );

    let bad_array_callable = runtime_error(
        r#"<?php
echo call_user_func_array(array("ClassName"), array());
"#,
    );
    assert_eq!(bad_array_callable.line, 2);
    assert_eq!(bad_array_callable.column, 6);
    assert_eq!(
        bad_array_callable.message,
        "unsupported call call_user_func_array(): array callback must be [object-or-class, method] in the current subset"
    );
}

#[test]
fn emit_ir_rejects_call_user_func_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho call_user_func('strlen', 'abc');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let metadata = emit_ir_source(
        r#"<?php
echo function_exists("call_user_func_array") ? "1" : "0";
echo is_callable("call_user_func_array") ? "1" : "0";
"#,
    )
    .unwrap();
    assert_eq!(metadata.matches("c\"1\\00\"").count(), 2, "{metadata}");

    let error =
        emit_ir_source("<?php\necho call_user_func_array('strlen', ['abc']);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
