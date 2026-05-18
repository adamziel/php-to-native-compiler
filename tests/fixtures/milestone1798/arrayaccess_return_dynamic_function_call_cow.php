<?php
function &milestone1798_pick(&$value) {
    return $value;
}

class Milestone1798_ArrayBag implements ArrayAccess {
    public $items = [];
    public $log = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $fn = "milestone1798_pick";
        $this->log[] = "get:" . $offset;
        return $fn($this->items[$offset]);
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

$source = "arrayaccess-dynamic-function-seed";
$node = ["value" => &$source, "plain" => ["value" => "arrayaccess-dynamic-function-copy"]];

$bag = new Milestone1798_ArrayBag();
$bag["slot"] = $node;
$bag["slot"]["value"] = "arrayaccess-dynamic-function";
$bag["slot"]["plain"]["value"] = "arrayaccess-dynamic-function-plain";

echo $source,
    "|",
    $bag->items["slot"]["plain"]["value"],
    "|",
    $bag->log[0],
    "|",
    $bag->log[1];
