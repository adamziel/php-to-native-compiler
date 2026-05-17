<?php
function &wp_refcow_pick_slot(&$items, $key, $suffix) {
    $items[$key] = $items[$key] . ":" . $suffix;
    return $items[$key];
}

class WP_RefCow_Slot_Picker {
    public $cache = [];

    public function &pick(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }

    public static function &pick_static(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }
}

$_REQUEST["payload"] = ["slot" => "request"];
$request_alias =& wp_refcow_pick_slot($_REQUEST["payload"], "slot", "function");
$request_alias = $request_alias . ":alias";
echo $_REQUEST["payload"]["slot"], "|", $request_alias, "\n";

$items = ["outer" => ["slot" => "array"]];
$array_alias =& WP_RefCow_Slot_Picker::pick_static($items["outer"], "slot", "static");
$array_alias = $array_alias . ":alias";
echo $items["outer"]["slot"], "|", $array_alias, "\n";

$picker = new WP_RefCow_Slot_Picker();
$picker->cache["options"]["alloptions"] = "cold";
$cache_alias =& $picker->pick($picker->cache["options"], "alloptions", "method");
$cache_alias = $cache_alias . ":alias";
echo $picker->cache["options"]["alloptions"], "|", $cache_alias;
