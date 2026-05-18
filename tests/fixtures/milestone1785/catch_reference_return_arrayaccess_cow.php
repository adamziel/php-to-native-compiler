<?php
class Milestone1785_ArrayBag implements ArrayAccess {
    public $items = [];
    public $log = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        try {
            throw new Exception();
        } catch (Exception $e) {
            $this->log[] = get_class($e);
            if (!isset($this->items[$offset])) {
                $this->items[$offset] = [];
            }
            return $this->items[$offset];
        }
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

$source = "arrayaccess-seed";
$node = ["value" => &$source, "plain" => ["value" => "arrayaccess-copy"]];

$bag = new Milestone1785_ArrayBag();
$bag["slot"] = $node;
$bag["slot"]["value"] = "arrayaccess-caught";
$bag["slot"]["plain"]["value"] = "arrayaccess-plain-caught";

echo $source, "|", $bag->items["slot"]["plain"]["value"], "|", $bag->log[0];
