<?php
error_reporting(0);

class Milestone2101_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get:" . ($offset === null ? "null" : $offset);
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->log[] = "set:" . ($offset === null ? "null" : $offset);
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$ref = "old";
$bag = new Milestone2101_Bag();
$bag[] = array("ref" => &$ref, "plain" => "plain-original");
$bag->store[""]["ref"] = "new";

echo implode(",", $bag->log), "|", $ref, "|", $bag->store[""]["plain"];
