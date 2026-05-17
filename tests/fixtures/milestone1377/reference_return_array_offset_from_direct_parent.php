<?php
function &wp_refcow_pick_direct_parent(&$items, $key, $suffix) {
    $items[$key] = $items[$key] . ":" . $suffix;
    return $items[$key];
}

class WP_RefCow_Direct_Parent_Picker {
    public function &pick(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }

    public static function &pick_static(&$items, $key, $suffix) {
        $items[$key] = $items[$key] . ":" . $suffix;
        return $items[$key];
    }
}

$function_items = ["slot" => "function"];
$function_alias =& wp_refcow_pick_direct_parent($function_items, "slot", "direct");
$function_alias = $function_alias . ":alias";
echo $function_items["slot"], "|", $function_alias, "\n";

$static_items = ["slot" => "static"];
$static_alias =& WP_RefCow_Direct_Parent_Picker::pick_static($static_items, "slot", "direct");
$static_alias = $static_alias . ":alias";
echo $static_items["slot"], "|", $static_alias, "\n";

$method_items = ["slot" => "method"];
$picker = new WP_RefCow_Direct_Parent_Picker();
$method_alias =& $picker->pick($method_items, "slot", "direct");
$method_alias = $method_alias . ":alias";
echo $method_items["slot"], "|", $method_alias, "\n";

$shared_items = ["slot" => "shared"];
$shared_parent =& $shared_items;
$shared_alias =& wp_refcow_pick_direct_parent($shared_items, "slot", "direct");
$shared_parent["slot"] = $shared_parent["slot"] . ":parent";
echo $shared_items["slot"], "|", $shared_parent["slot"], "|", $shared_alias;
