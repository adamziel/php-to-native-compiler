<?php
class WP_RefCow_Nested_ArrayAccess_Bag implements ArrayAccess {
    private $items = ["outer" => ["slot" => "seed"]];

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

function wp_refcow_nested_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_nested_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$bag = new WP_RefCow_Nested_ArrayAccess_Bag();

$alias =& $bag["outer"]["slot"];
$alias = $alias . ":alias";
echo $bag["outer"]["slot"], "|", $alias, "\n";

echo call_user_func_array("wp_refcow_nested_mark", array(&$bag["outer"]["slot"], "callback")), "|", $bag["outer"]["slot"], "\n";

$created =& call_user_func_array("wp_refcow_nested_pick", array(&$bag["created"]["leaf"], "return"));
$created = $created . ":alias";
echo $bag["created"]["leaf"], "|", $created;
