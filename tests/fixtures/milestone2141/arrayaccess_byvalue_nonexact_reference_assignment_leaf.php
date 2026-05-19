<?php
error_reporting(0);

class Milestone2141_Bag implements ArrayAccess {
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

$bag = new Milestone2141_Bag();
$bag->store["slot"] = array("ref" => "old", "plain" => "plain");
$source =& $bag->store["slot"]["ref"];

$alias =& $bag["slot"]["ref"];
$alias = "new";

echo implode(",", $bag->log), "|", $source, "|", $bag->store["slot"]["ref"], "|", $alias, "|", $bag->store["slot"]["plain"];
