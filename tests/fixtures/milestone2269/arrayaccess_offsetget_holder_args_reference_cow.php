<?php
error_reporting(0);

function &milestone2269_pick(&$value) {
    return $value["ref"]["value"];
}

class Milestone2269_Holder implements ArrayAccess {
    public $store = array();

    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
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

class Milestone2269_Bag implements ArrayAccess {
    public $store = array();
    public $holder;

    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        $this->holder["args"] = array($bucket);
        $alias =& call_user_func_array("milestone2269_pick", $this->holder["args"]);
        $alias = "inside";
        return $bucket;
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

$bag = new Milestone2269_Bag();
$bag->holder = new Milestone2269_Holder();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $bag->store["slot"]["ref"]["value"];

$copy = $bag["slot"];
echo $ref, "|", $copy["ref"]["value"], "|", $copy["plain"]["value"], "|", $bag->holder->store["args"][0]["ref"]["value"];
