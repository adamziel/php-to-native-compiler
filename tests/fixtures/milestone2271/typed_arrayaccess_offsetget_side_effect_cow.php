<?php
error_reporting(0);

class Milestone2271_Bag implements ArrayAccess {
    public $store = array();
    public $hits = 0;

    public function offsetExists(mixed $offset): bool {
        return isset($this->store[$offset]);
    }

    public function offsetGet(mixed $offset): mixed {
        $this->hits = $this->hits + 1;
        $key = $offset;
        return $this->store[$key];
    }

    public function offsetSet(mixed $offset, mixed $value): void {
        $this->store[$offset] = $value;
    }

    public function offsetUnset(mixed $offset): void {
        unset($this->store[$offset]);
    }
}

$bag = new Milestone2271_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $bag->store["slot"]["ref"]["value"];

$bag["slot"]["ref"]["value"] = "inside";
$bag["slot"]["plain"]["value"] = "copy-only";

echo $ref, "|", $bag->store["slot"]["plain"]["value"], "|", $bag->hits;
