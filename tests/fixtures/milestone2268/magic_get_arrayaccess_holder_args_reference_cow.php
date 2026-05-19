<?php
error_reporting(0);

function &milestone2268_pick(&$value) {
    return $value["ref"]["value"];
}

class Milestone2268_Holder implements ArrayAccess {
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

class Milestone2268_Box {
    public $store = array();
    public $holder;

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->holder["args"] = array($bucket);
        $alias =& call_user_func_array("milestone2268_pick", $this->holder["args"]);
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2268_Box();
$box->holder = new Milestone2268_Holder();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $copy["ref"]["value"], "|", $copy["plain"]["value"], "|", $box->holder->store["args"][0]["ref"]["value"];
