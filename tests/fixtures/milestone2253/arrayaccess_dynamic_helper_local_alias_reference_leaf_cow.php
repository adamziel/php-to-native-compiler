<?php
error_reporting(0);

class Milestone2253_Box implements ArrayAccess {
    public $store = array();

    public function mutate($bucket) {
        $alias =& $bucket["ref"]["value"];
        $alias = "copy-ref";
        $bucket["plain"]["value"] = "copy-plain";
        return $bucket;
    }

    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        $method = "mutate";
        return $this->{$method}($bucket);
    }

    public function offsetSet($offset, $value) {
    }

    public function offsetExists($offset) {
        return true;
    }

    public function offsetUnset($offset) {
    }
}

$box = new Milestone2253_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box["slot"];
echo $ref, "|", $box->store["slot"]["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
