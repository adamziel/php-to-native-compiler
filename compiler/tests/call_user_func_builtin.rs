use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
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
fn call_user_func_array_binds_array_access_reference_roots() {
    let execution = run_source(
        r#"<?php
class WP_RefCow_ArrayAccess_Bag implements ArrayAccess {
    private $items = ["slot" => "seed"];

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

$bag = new WP_RefCow_ArrayAccess_Bag();
$alias =& $bag["slot"];
$alias = $alias . ":alias";
echo $bag["slot"], "|", $alias, "\n";

echo call_user_func_array("wp_refcow_array_access_mark", array(&$bag["slot"], "callback")), "|", $bag["slot"], "\n";

$picked =& call_user_func_array("wp_refcow_array_access_pick", array(&$bag["missing"], "return"));
$picked = $picked . ":picked";
echo $bag["missing"], "|", $picked;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:alias|seed:alias\nseed:alias:callback|seed:alias:callback\nnull:return:picked|null:return:picked"
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

    let array_callable = runtime_error(
        r#"<?php
echo call_user_func(["ClassName", "method"]);
"#,
    );
    assert_eq!(array_callable.line, 2);
    assert_eq!(array_callable.column, 6);
    assert_eq!(
        array_callable.message,
        "unsupported call call_user_func(): array callables are not implemented in the current subset"
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

    let non_array_args = runtime_error(
        r#"<?php
echo call_user_func_array("strlen", "four");
"#,
    );
    assert_eq!(non_array_args.line, 2);
    assert_eq!(non_array_args.column, 6);
    assert_eq!(
        non_array_args.message,
        "unsupported call call_user_func_array(): argument array must be array in the current subset, got string"
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
