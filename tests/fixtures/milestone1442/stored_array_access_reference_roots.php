<?php
class WP_RefCow_Stored_ArrayAccess_Bag implements ArrayAccess {
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

class WP_RefCow_Stored_ArrayAccess_Holder {
    public $bag;
}

function wp_refcow_stored_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_stored_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$bag = new WP_RefCow_Stored_ArrayAccess_Bag();

$args = [];
$args[0] =& $bag["slot"];
$args[1] = "stored";
call_user_func_array("wp_refcow_stored_array_access_mark", $args);
echo $bag["slot"], "|", $args[0], "\n";

$nested = [];
$nested["value"] =& $bag["outer"]["slot"];
$nested["suffix"] = "nested";
$picked =& call_user_func_array("wp_refcow_stored_array_access_pick", $nested);
$picked = $picked . ":alias";
echo $bag["outer"]["slot"], "|", $nested["value"], "|", $picked, "\n";

$holder = new WP_RefCow_Stored_ArrayAccess_Holder();
$holder->bag = new WP_RefCow_Stored_ArrayAccess_Bag();

$held = [];
$held[] =& $holder->bag["created"]["leaf"];
$held[] = "held";
call_user_func_array("wp_refcow_stored_array_access_mark", $held);
echo $holder->bag["created"]["leaf"], "|", $held[0];
