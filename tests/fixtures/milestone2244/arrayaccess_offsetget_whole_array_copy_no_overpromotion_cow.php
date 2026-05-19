<?php
error_reporting(0);

function &milestone2244_whole(&$value) {
    return $value;
}

class Milestone2244_Holder {
    public $args = array();
}

class Milestone2244_Box implements ArrayAccess {
    public $store = array();
    public $holder;

    public function offsetGet($name) {
        $bucket = $this->store[$name];
        $this->holder->args = array($bucket);
        $alias =& call_user_func_array("milestone2244_whole", $this->holder->args);
        $alias["plain"]["value"] = "copy-only";
        return $bucket;
    }

    public function offsetSet($name, $value) {
        $this->store[$name] = $value;
    }

    public function offsetExists($name) {
        return isset($this->store[$name]);
    }

    public function offsetUnset($name) {
        unset($this->store[$name]);
    }
}

$box = new Milestone2244_Box();
$box->holder = new Milestone2244_Holder();
$box->store = array(
    "slot" => array(
        "plain" => array("value" => "plain"),
    ),
);

$copy = $box["slot"];
echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
