<?php
error_reporting(0);

function &milestone2256_pick_leaf(&$value) {
    return $value["ref"]["value"];
}

function milestone2256_make_holder($copy) {
    $holder = new stdClass();
    $holder->args = array($copy);
    return $holder;
}

class Milestone2256_Box implements ArrayAccess {
    public $store = array();

    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        $holder = milestone2256_make_holder($bucket);
        $alias =& call_user_func_array("milestone2256_pick_leaf", $holder->args);
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

$box = new Milestone2256_Box();
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
