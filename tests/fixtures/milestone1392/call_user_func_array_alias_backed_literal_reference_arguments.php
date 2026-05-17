<?php
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
