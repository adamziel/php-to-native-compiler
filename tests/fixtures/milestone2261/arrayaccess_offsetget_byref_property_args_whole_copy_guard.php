<?php
error_reporting(0);

function &milestone2261_whole(&$value) {
    return $value;
}

class Milestone2261_Box implements ArrayAccess {
    public $store = array();
    public $args = array();

    public function prepare($copy, &$args) {
        $args = array($copy);
    }

    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        $this->args = array();
        $this->prepare($bucket, $this->args);
        $alias =& call_user_func_array("milestone2261_whole", $this->args);
        $alias["plain"]["value"] = "copy-only";
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

$box = new Milestone2261_Box();
$box->store = array(
    "slot" => array("plain" => array("value" => "plain")),
);

$copy = $box["slot"];
echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
