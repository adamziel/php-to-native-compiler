<?php
error_reporting(0);

class Milestone2146_Source {
    public $store = array();
}

class Milestone2146_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        $key = $offset;
        if ($key === "slot") {
            $bucket = $this->store[$key];
        } else {
            $bucket = array();
        }
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->log[] = "set:" . $offset;
        $key = $offset;
        if ($key === "slot") {
            $payload = $value;
        } else {
            $payload = array();
        }
        $this->store[$key] = $payload;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone2146_Holder {
    public $bag;
}

$source = new Milestone2146_Source();
$source->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $source->store["slot"]["ref"];

$holder = new Milestone2146_Holder();
$holder->bag = new Milestone2146_Bag();
$property = "bag";

$holder->$property["slot"] = $source->store["slot"];
$holder->$property["slot"]["ref"] = "new";
$holder->$property["slot"]["plain"] = "copy";

echo implode(",", $holder->bag->log), "|", $alias, "|", $source->store["slot"]["ref"], "|", $holder->bag->store["slot"]["plain"];
