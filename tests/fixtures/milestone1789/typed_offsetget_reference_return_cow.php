<?php
class Milestone1789_ArrayBag implements ArrayAccess {
    public $items = [];
    public $log = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    public function &offsetGet(mixed $offset): mixed {
        $this->log[] = "typed:" . $offset;
        if (!isset($this->items[$offset])) {
            $this->items[$offset] = [];
        }
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

$source = "typed-offset-seed";
$node = ["value" => &$source, "plain" => ["value" => "typed-offset-copy"]];

$bag = new Milestone1789_ArrayBag();
$bag["slot"] = $node;
$bag["slot"]["value"] = "typed-offset";
$bag["slot"]["plain"]["value"] = "typed-offset-plain";

echo $source, "|", $bag->items["slot"]["plain"]["value"], "|", $bag->log[0];
