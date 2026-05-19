<?php
error_reporting(0);

class Milestone2136_Bag implements ArrayAccess {
    private $items;
    public $log = array();

    public function seed(&$source) {
        $this->items = array("box" => array("leaf" => &$source));
    }

    public function &box() {
        return $this->items["box"];
    }

    public function offsetSet($offset, $value) {
        $this->log[] = "set:" . $offset . ":" . $value;
        $this->items = array();
    }

    public function offsetExists($offset) {
        return isset($this->items[$offset]);
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
$bag = new Milestone2136_Bag();
$bag->seed($source);
$alias =& $bag->box();
$bag["slot"] = 1;
$alias["leaf"] = "mutated";

echo implode(",", $bag->log), "|", $source, "|", $alias["leaf"], "|", $bag->size();
