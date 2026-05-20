<?php
error_reporting(0);

class Milestone2280_Box implements ArrayAccess {
    public $store = array();

    public function keep(&$bucket) {
    }

    public function offsetGet($offset) {
        $bucket = $this->store;
        $this->keep($bucket);
        return $bucket[$offset];
    }

    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$box = new Milestone2280_Box();
$box->store = array("slot" => array("plain" => array("value" => "plain")));

$copy = $box["slot"];
$copy["plain"]["value"] = "copy-only";

echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
