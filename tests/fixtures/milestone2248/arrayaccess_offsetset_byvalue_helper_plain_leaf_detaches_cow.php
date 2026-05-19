<?php
error_reporting(0);

function milestone2248_key($key) {
    return $key;
}

class Milestone2248_Box implements ArrayAccess {
    public $store = array();

    public function offsetSet($offset, $value) {
        $key = milestone2248_key($offset);
        $this->store[$key] = $value;
    }

    public function offsetGet($offset) {
        return $this->store[$offset];
    }

    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$copy = array("plain" => array("value" => "plain"));
$box = new Milestone2248_Box();
$box["slot"] = $copy;
$box["slot"]["plain"]["value"] = "changed";

echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
