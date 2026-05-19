<?php
error_reporting(0);

class Milestone2152_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get";
        $value = $this->store[$offset];
        $this->store = array($offset => array("ref" => "new"));
        return $value;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2152_Bag();
$bag->store["slot"] = array("ref" => "old");
$alias =& $bag->store["slot"]["ref"];

$value = $bag["slot"];
$value["ref"] = "value";
$alias = "alias";

echo implode(",", $bag->log), "|", $alias, "|", $value["ref"], "|", $bag->store["slot"]["ref"];
