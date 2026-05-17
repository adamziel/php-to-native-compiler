<?php
function &wp_refcow_pick_normal_slot(&$items, $key, $suffix) {
    $items[$key] = $items[$key] . ":" . $suffix;
    return $items[$key];
}

class WP_RefCow_Normal_Array_Offset_Picker {
    public $cache = [];

    public function &pick(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }

    public static function &pickStatic(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }
}

$_REQUEST["payload"] = ["slot" => "request"];
$payload =& $_REQUEST["payload"];
echo wp_refcow_pick_normal_slot($payload, "slot", "function"), "|", $_REQUEST["payload"]["slot"], "\n";

$items = ["outer" => ["slot" => "array"]];
echo WP_RefCow_Normal_Array_Offset_Picker::pickStatic($items["outer"], "slot", "static"), "|", $items["outer"]["slot"], "\n";

$picker = new WP_RefCow_Normal_Array_Offset_Picker();
$picker->cache["options"]["alloptions"] = "cold";
echo $picker->pick($picker->cache["options"], "alloptions", "method"), "|", $picker->cache["options"]["alloptions"], "\n";

$dynamic = ["slot" => "dynamic"];
$class = "WP_RefCow_Normal_Array_Offset_Picker";
echo $class::pickStatic($dynamic, "slot", "dynamic"), "|", $dynamic["slot"];
