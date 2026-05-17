<?php
class WP_RefCow_Append_ArrayAccess_Bag implements ArrayAccess {
    public $items = [];

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

class WP_RefCow_Append_ArrayAccess_Holder {
    public $bag;
}

function wp_refcow_append_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_append_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$bag = new WP_RefCow_Append_ArrayAccess_Bag();
$direct =& $bag[];
$direct = "direct";
echo $bag->items[""], "|", $direct, "\n";

$args = [];
$args[0] =& $bag[];
$args[1] = "stored";
call_user_func_array("wp_refcow_append_array_access_mark", $args);
echo $bag->items[""], "|", $args[0], "\n";

$alias =& call_user_func_array("wp_refcow_append_array_access_pick", $args);
$alias = $alias . ":alias";
echo $bag->items[""], "|", $args[0], "|", $alias, "\n";

$holder = new WP_RefCow_Append_ArrayAccess_Holder();
$holder->bag = new WP_RefCow_Append_ArrayAccess_Bag();
$held = [];
$held["value"] =& $holder->bag[];
$held["suffix"] = "held";
call_user_func_array("wp_refcow_append_array_access_mark", $held);
echo $holder->bag->items[""], "|", $held["value"];
