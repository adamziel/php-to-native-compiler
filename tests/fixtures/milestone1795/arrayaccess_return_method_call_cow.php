<?php
class Milestone1795_ArrayBag implements ArrayAccess {
    public $items = [];
    public $log = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    public function &slotRef($offset) {
        $this->log[] = "slotRef:" . $offset;
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        return $this->slotRef($offset);
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

$source = "arrayaccess-method-seed";
$node = ["value" => &$source, "plain" => ["value" => "arrayaccess-method-copy"]];

$bag = new Milestone1795_ArrayBag();
$bag["slot"] = $node;
$bag["slot"]["value"] = "arrayaccess-method";
$bag["slot"]["plain"]["value"] = "arrayaccess-method-plain";

echo $source,
    "|",
    $bag->items["slot"]["plain"]["value"],
    "|",
    $bag->log[0],
    "|",
    $bag->log[1],
    "|",
    $bag->log[2],
    "|",
    $bag->log[3];
