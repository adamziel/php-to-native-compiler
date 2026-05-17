<?php
function &wp_refcow_pick_alias_backed_parent(&$payload, $key, $suffix) {
    $payload[$key] = $payload[$key] . ":" . $suffix;
    return $payload[$key];
}

class WP_RefCow_Alias_Backed_Parent_Picker {
    public function &pick(&$payload, $key, $suffix) {
        $payload[$key] = $payload[$key] . ":" . $suffix;
        return $payload[$key];
    }

    public static function &pick_static(&$payload, $key, $suffix) {
        $payload[$key] = $payload[$key] . ":" . $suffix;
        return $payload[$key];
    }
}

$_REQUEST["payload"] = ["slot" => "request"];
$request_payload =& $_REQUEST["payload"];
$request_alias =& wp_refcow_pick_alias_backed_parent($request_payload, "slot", "function");
$request_alias = $request_alias . ":alias";
echo $_REQUEST["payload"]["slot"], "|", $request_payload["slot"], "|", $request_alias, "\n";

$items = ["outer" => ["slot" => "array"]];
$outer =& $items["outer"];
$static_alias =& WP_RefCow_Alias_Backed_Parent_Picker::pick_static($outer, "slot", "static");
$static_alias = $static_alias . ":alias";
echo $items["outer"]["slot"], "|", $outer["slot"], "|", $static_alias, "\n";

$method_items = ["outer" => ["slot" => "method"]];
$method_parent =& $method_items["outer"];
$picker = new WP_RefCow_Alias_Backed_Parent_Picker();
$method_alias =& $picker->pick($method_parent, "slot", "method");
$method_items["outer"]["slot"] = $method_items["outer"]["slot"] . ":root";
echo $method_items["outer"]["slot"], "|", $method_parent["slot"], "|", $method_alias;
