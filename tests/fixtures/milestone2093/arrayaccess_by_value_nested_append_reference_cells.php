<?php
error_reporting(0);

class Milestone2093_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = array(
            "ref" => &$this->store[$offset]["ref"],
            "plain" => $this->store[$offset]["plain"],
        );
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2093_Bag();
$ref = array("seed");
$bag->store["slot"] = array("ref" => &$ref, "plain" => array("plain-original"));

$bag["slot"]["ref"][] = "copy";
$bag["slot"]["plain"][] = "plain-copy";

echo $ref[1], "|", count($bag->store["slot"]["plain"]);
