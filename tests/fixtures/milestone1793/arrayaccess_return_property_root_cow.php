<?php
class Milestone1793_ArrayBag implements ArrayAccess {
    public $items = [];
    public $log = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        return $this->items;
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

$source = "arrayaccess-root-seed";
$node = ["value" => &$source, "plain" => ["value" => "arrayaccess-root-copy"]];

$bag = new Milestone1793_ArrayBag();
$bag["slot"] = $node;
$bag["ignored"]["slot"]["value"] = "arrayaccess-root";
$bag["ignored"]["slot"]["plain"]["value"] = "arrayaccess-root-plain";

echo $source,
    "|",
    $bag->items["slot"]["plain"]["value"],
    "|",
    $bag->log[0],
    "|",
    $bag->log[1];
