<?php
error_reporting(0);

class Milestone2137_Bag implements ArrayAccess {
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
        return isset($this->items[$offset]);
    }

    public function offsetGet($offset) {
        return $this->items[$offset];
    }

    public function offsetUnset($offset) {
        $this->log[] = "unset:" . $offset;
        $this->items = array();
    }

    public function size() {
        return count($this->items);
    }
}

$source = "seed";
$bag = new Milestone2137_Bag();
$bag->seed($source);
$alias =& $bag->box();
unset($bag["slot"]);
$alias["leaf"] = "mutated";

echo implode(",", $bag->log), "|", $source, "|", $alias["leaf"], "|", $bag->size();
