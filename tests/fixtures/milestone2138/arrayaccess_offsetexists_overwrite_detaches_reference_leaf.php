<?php
error_reporting(0);

class Milestone2138_Bag implements ArrayAccess {
    private $items;
    public $log = array();

    public function seed(&$source) {
        $this->items = array("box" => array("leaf" => &$source));
    }

    public function &box() {
        return $this->items["box"];
    }

    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    public function offsetExists($offset) {
        $this->log[] = "exists:" . $offset;
        $this->items = array();
        return false;
    }

    public function offsetGet($offset) {
        return $this->items[$offset];
    }

    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }

    public function size() {
        return count($this->items);
    }
}

$source = "seed";
$bag = new Milestone2138_Bag();
$bag->seed($source);
$alias =& $bag->box();
$seen = isset($bag["box"]);
$alias["leaf"] = "mutated";

echo implode(",", $bag->log), "|", ($seen ? "yes" : "no"), "|", $source, "|", $alias["leaf"], "|", $bag->size();
