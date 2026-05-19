<?php
error_reporting(0);

class Milestone2176_Bag implements ArrayAccess {
    public $store = array();

    public function load($offset) {
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $method = new ReflectionMethod($this, "load");
        return $method->invoke($this, $offset);
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$source = array("slot" => array("ref" => "old", "plain" => "plain"));
$alias =& $source["slot"]["ref"];

$bag = new Milestone2176_Bag();
$bag->store["slot"] = $source["slot"];

$value = $bag["slot"];
$value["ref"] = "new";
$value["plain"] = "copy";

echo $alias,
    "|",
    $source["slot"]["ref"],
    "|",
    $value["ref"],
    "|",
    $source["slot"]["plain"],
    "|",
    $value["plain"];
