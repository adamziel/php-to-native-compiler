<?php
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
