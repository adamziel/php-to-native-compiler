<?php
error_reporting(0);

class Milestone2174_Bag implements ArrayAccess {
    public $store = array();
    public $cb;

    public function __construct() {
        $this->cb = array($this, "load");
    }

    public function load($offset) {
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $cb = $this->cb;
        return $cb($offset);
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

$bag = new Milestone2174_Bag();
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
