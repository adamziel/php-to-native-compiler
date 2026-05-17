<?php
class WP_RefCow_NonDirect_Foreach_ArrayAccess_Bag implements ArrayAccess {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];

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

class WP_RefCow_NonDirect_Foreach_ArrayAccess_Holder {
    public $bag;
    public $dynamicBag;
}

$holders = [];
$primary = new WP_RefCow_NonDirect_Foreach_ArrayAccess_Holder();
$primary->bag = new WP_RefCow_NonDirect_Foreach_ArrayAccess_Bag();
$holders["primary"] = $primary;
$bag = $holders["primary"]->bag;
foreach ($holders["primary"]->bag["outer"] as $key => &$value) {
    $value = "array:" . $key;
    if ($key === "a") {
        $bag->items["outer"]["c"] = "array-c";
    }
}

echo $holders["primary"]->bag["outer"]["a"], "|", $holders["primary"]->bag["outer"]["b"], "|", $value, "\n";
$bag->items["outer"]["c"] = "array-direct";
echo $value, "|";
$value = "array-tail";
echo $holders["primary"]->bag["outer"]["c"], "|", $value, "\n";
unset($value);

$dynamic = new WP_RefCow_NonDirect_Foreach_ArrayAccess_Holder();
$dynamic->dynamicBag = new WP_RefCow_NonDirect_Foreach_ArrayAccess_Bag();
$holders["dynamic"] = $dynamic;
$dynamicBag = $holders["dynamic"]->dynamicBag;
$property = "dynamicBag";
foreach ($holders["dynamic"]->{$property}["outer"] as $key => &$value) {
    $value = "dynamic:" . $key;
    if ($key === "a") {
        $dynamicBag->items["outer"]["c"] = "dynamic-c";
    }
}

echo $holders["dynamic"]->dynamicBag["outer"]["a"], "|", $holders["dynamic"]->dynamicBag["outer"]["b"], "|", $value, "\n";
$dynamicBag->items["outer"]["c"] = "dynamic-direct";
echo $value, "|";
$value = "dynamic-tail";
echo $holders["dynamic"]->dynamicBag["outer"]["c"], "|", $value;
