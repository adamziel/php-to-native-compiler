<?php
class Milestone1753_Box {
    public int $id = 1;
}

class Milestone1753_MagicBox {
    private $store = array();

    public function seed(&$value) {
        $this->store["missing"]["copy"] =& $value;
    }

    public function &__get($name) {
        $bucket =& $this->store[$name];
        return $bucket;
    }

    public function read($name, $key) {
        return gettype($this->store[$name][$key]) . ":" . $this->store[$name][$key];
    }
}

$box = new Milestone1753_Box();
$alias =& $box->id;
$items = array();
$items["slot"] =& $alias;
$slot =& $items["slot"];
$slot = "2";

$magic = new Milestone1753_MagicBox();
$fn = function () use (&$slot, $magic) {
    $magic->seed($slot);
};
$fn();

$magic->missing["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", $magic->read("missing", "copy"), "|", gettype($slot), ":", $slot;
