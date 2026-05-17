<?php
class WP_RefCow_Property_ArrayAccess_Bag implements ArrayAccess {
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

class WP_RefCow_Property_ArrayAccess_Holder {
    public $bag;
}

function wp_refcow_property_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_property_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$holder = new WP_RefCow_Property_ArrayAccess_Holder();
$holder->bag = new WP_RefCow_Property_ArrayAccess_Bag();

$alias =& $holder->bag["outer"]["slot"];
$alias = $alias . ":alias";
echo $holder->bag["outer"]["slot"], "|", $alias, "\n";

echo call_user_func_array("wp_refcow_property_array_access_mark", array(&$holder->bag["outer"]["slot"], "callback")), "|", $holder->bag["outer"]["slot"], "\n";

$created =& call_user_func_array("wp_refcow_property_array_access_pick", array(&$holder->bag["created"]["leaf"], "return"));
$created = $created . ":alias";
echo $holder->bag["created"]["leaf"], "|", $created;
