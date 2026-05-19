<?php
error_reporting(0);

class Milestone2161_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get";
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $store = function($key, $payload) {
            $this->log[] = "set";
            $this->store[$key] = $payload;
        };
        $store($offset, $value);
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$source = array("slot" => array("ref" => "old", "plain" => "plain"));
$alias =& $source["slot"]["ref"];

$bag = new Milestone2161_Bag();
$bag["slot"] = $source["slot"];

$value = $bag["slot"];
$value["ref"] = "new";
$value["plain"] = "copy";

echo implode(",", $bag->log), "|", $alias, "|", $source["slot"]["ref"], "|", $bag->store["slot"]["ref"], "|", $bag->store["slot"]["plain"], "|", $value["plain"];
