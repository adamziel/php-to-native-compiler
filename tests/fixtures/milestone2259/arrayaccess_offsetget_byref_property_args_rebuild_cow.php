<?php
error_reporting(0);

function &milestone2259_pick_leaf(&$value) {
    return $value["ref"]["value"];
}

class Milestone2259_Box implements ArrayAccess {
    public $store = array();
    public $args = array();

    public function prepare($copy, &$args) {
        $args = array($copy);
    }

    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        $this->args = array();
        $this->prepare($bucket, $this->args);
        $alias =& call_user_func_array("milestone2259_pick_leaf", $this->args);
        $alias = "inside";
        return $bucket;
    }

    public function offsetSet($offset, $value) {
    }

    public function offsetExists($offset) {
        return true;
    }

    public function offsetUnset($offset) {
    }
}

$box = new Milestone2259_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box["slot"];
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
