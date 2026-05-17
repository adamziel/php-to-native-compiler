<?php
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

echo call_user_func_array(
    "wp_refcow_dynamic_property_array_access_mark",
    array(&$holder->{$name}["outer"]["slot"], "callback")
), "|", $holder->bag["outer"]["slot"], "\n";

$stored = [];
$stored["value"] =& $holder->{$name}["created"]["leaf"];
$stored["suffix"] = "stored";
call_user_func_array("wp_refcow_dynamic_property_array_access_mark", $stored);
echo $holder->bag["created"]["leaf"], "|", $stored["value"], "\n";

$picked =& call_user_func_array(
    "wp_refcow_dynamic_property_array_access_pick",
    array(&$holder->{$name}["return"], "pick")
);
$picked = $picked . ":picked";
echo $holder->bag["return"], "|", $picked;
