<?php
class WP_RefCow_Foreach_ArrayAccess_Bag implements ArrayAccess {
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

class WP_RefCow_Foreach_ArrayAccess_Holder {
    public $bag;
}

$bag = new WP_RefCow_Foreach_ArrayAccess_Bag();
foreach ($bag["outer"] as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "a") {
        $bag->items["outer"]["c"] = "three";
    }
}

echo $bag["outer"]["a"], "|", $bag["outer"]["b"], "|", $value, "\n";
$bag->items["outer"]["c"] = "direct";
echo $value, "|";
$value = "tail";
echo $bag["outer"]["c"], "|", $value, "\n";
unset($value);

$namedHolder = new WP_RefCow_Foreach_ArrayAccess_Holder();
$namedHolder->bag = new WP_RefCow_Foreach_ArrayAccess_Bag();
$namedBag = $namedHolder->bag;
foreach ($namedHolder->bag["outer"] as $key => &$value) {
    $value = "named:" . $key;
    if ($key === "a") {
        $namedBag->items["outer"]["c"] = "named-c";
    }
}

echo $namedHolder->bag["outer"]["a"], "|", $namedHolder->bag["outer"]["b"], "|", $value, "\n";
$namedBag->items["outer"]["c"] = "named-direct";
echo $value, "|";
$value = "named-tail";
echo $namedHolder->bag["outer"]["c"], "|", $value, "\n";
unset($value);

$holder = new WP_RefCow_Foreach_ArrayAccess_Holder();
$holder->bag = new WP_RefCow_Foreach_ArrayAccess_Bag();
$heldBag = $holder->bag;
$property = "bag";
foreach ($holder->{$property}["outer"] as $key => &$value) {
    $value = "held:" . $key;
    if ($key === "a") {
        $heldBag->items["outer"]["c"] = "held-c";
    }
}

echo $holder->bag["outer"]["a"], "|", $holder->bag["outer"]["b"], "|", $value, "\n";
$heldBag->items["outer"]["c"] = "held-direct";
echo $value, "|";
$value = "held-tail";
echo $holder->bag["outer"]["c"], "|", $value;
