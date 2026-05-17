<?php
class WP_RefCow_ArrayAccess_Bag implements ArrayAccess {
    private $items = ["slot" => "seed"];

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

function wp_refcow_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$bag = new WP_RefCow_ArrayAccess_Bag();
$alias =& $bag["slot"];
$alias = $alias . ":alias";
echo $bag["slot"], "|", $alias, "\n";

echo call_user_func_array("wp_refcow_array_access_mark", array(&$bag["slot"], "callback")), "|", $bag["slot"], "\n";

$picked =& call_user_func_array("wp_refcow_array_access_pick", array(&$bag["missing"], "return"));
$picked = $picked . ":picked";
echo $bag["missing"], "|", $picked;
