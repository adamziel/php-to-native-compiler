<?php
function &wp_refcow_pick_nested_direct_parent(&$items, $key, $subkey, $suffix) {
    $items[$key][$subkey] = $items[$key][$subkey] . ":" . $suffix;
    return $items[$key][$subkey];
}

class WP_RefCow_Nested_Direct_Parent_Picker {
    public function &pick(&$items, $key, $subkey, $suffix) {
        $items[$key][$subkey] = $items[$key][$subkey] . ":" . $suffix;
        return $items[$key][$subkey];
    }

    public static function &pick_static(&$items, $key, $subkey, $suffix) {
        $items[$key][$subkey] = $items[$key][$subkey] . ":" . $suffix;
        return $items[$key][$subkey];
    }
}

$function_items = ["outer" => ["slot" => "function"]];
$function_alias =& wp_refcow_pick_nested_direct_parent($function_items, "outer", "slot", "direct");
$function_alias = $function_alias . ":alias";
echo $function_items["outer"]["slot"], "|", $function_alias, "\n";

$static_items = ["outer" => ["slot" => "static"]];
$static_alias =& WP_RefCow_Nested_Direct_Parent_Picker::pick_static($static_items, "outer", "slot", "direct");
$static_alias = $static_alias . ":alias";
echo $static_items["outer"]["slot"], "|", $static_alias, "\n";

$method_items = ["outer" => ["slot" => "method"]];
$picker = new WP_RefCow_Nested_Direct_Parent_Picker();
$method_alias =& $picker->pick($method_items, "outer", "slot", "direct");
$method_alias = $method_alias . ":alias";
echo $method_items["outer"]["slot"], "|", $method_alias, "\n";

$shared_items = ["outer" => ["slot" => "shared"]];
$shared_parent =& $shared_items;
$shared_alias =& wp_refcow_pick_nested_direct_parent($shared_items, "outer", "slot", "direct");
$shared_parent["outer"]["slot"] = $shared_parent["outer"]["slot"] . ":parent";
echo $shared_items["outer"]["slot"], "|", $shared_parent["outer"]["slot"], "|", $shared_alias;
