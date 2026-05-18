<?php
class Milestone1788_CustomException extends Exception {}

class Milestone1788_ArrayBag implements ArrayAccess {
    public $items = [];
    public $log = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        try {
            throw new Milestone1788_CustomException();
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

$source = "subclass-seed";
$node = ["value" => &$source, "plain" => ["value" => "subclass-copy"]];

$bag = new Milestone1788_ArrayBag();
$bag["slot"] = $node;
$bag["slot"]["value"] = "subclass-caught";
$bag["slot"]["plain"]["value"] = "subclass-plain-caught";

echo $source, "|", $bag->items["slot"]["plain"]["value"], "|", $bag->log[0];
