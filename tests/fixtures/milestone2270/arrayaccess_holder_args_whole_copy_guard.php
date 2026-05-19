<?php
error_reporting(0);

function &milestone2270_whole(&$value) {
    return $value;
}

class Milestone2270_Holder implements ArrayAccess {
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

class Milestone2270_Box {
    public $store = array();
    public $holder;

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->holder["args"] = array($bucket);
        $alias =& call_user_func_array("milestone2270_whole", $this->holder["args"]);
        $alias["plain"]["value"] = "copy-only";
        return $bucket;
    }
}

$box = new Milestone2270_Box();
$box->holder = new Milestone2270_Holder();
$box->store = array(
    "slot" => array("plain" => array("value" => "plain")),
);

$copy = $box->slot;
echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], "|", $box->holder->store["args"][0]["plain"]["value"];
