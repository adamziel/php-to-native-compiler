<?php
error_reporting(0);

class Milestone2139_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        if ($offset === "slot") {
            $bucket = $this->store[$offset];
        } else {
            $bucket = array();
        }
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$bag = new Milestone2139_Bag();
$bag->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $bag->store["slot"]["ref"];

$bag["slot"]["ref"] = "new";
$bag["slot"]["plain"] = "copy";

echo implode(",", $bag->log), "|", $alias, "|", $bag->store["slot"]["ref"], "|", $bag->store["slot"]["plain"];
