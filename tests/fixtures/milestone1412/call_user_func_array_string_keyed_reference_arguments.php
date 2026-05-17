<?php
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
