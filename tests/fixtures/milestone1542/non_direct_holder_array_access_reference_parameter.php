<?php
class WP_RefCow_Reference_Parameter_ArrayAccess_Bag implements ArrayAccess {
    public $items = ["outer" => ["slot" => "seed"]];

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

class WP_RefCow_Reference_Parameter_ArrayAccess_Holder {
    public $bag;
    public $dynamicBag;
}

function wp_refcow_touch_slot(&$value) {
    $value = $value . ":touched";
}

$holders = [];
$primary = new WP_RefCow_Reference_Parameter_ArrayAccess_Holder();
$primary->bag = new WP_RefCow_Reference_Parameter_ArrayAccess_Bag();
$holders["primary"] = $primary;
wp_refcow_touch_slot($holders["primary"]->bag["outer"]["slot"]);
echo $primary->bag["outer"]["slot"], "\n";

$dynamic = new WP_RefCow_Reference_Parameter_ArrayAccess_Holder();
$dynamic->dynamicBag = new WP_RefCow_Reference_Parameter_ArrayAccess_Bag();
$holders["dynamic"] = $dynamic;
$property = "dynamicBag";
wp_refcow_touch_slot($holders["dynamic"]->{$property}["outer"]["slot"]);
echo $dynamic->dynamicBag["outer"]["slot"];
