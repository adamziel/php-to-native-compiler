<?php
class Milestone1881_Bag implements ArrayAccess {
    public $which = "store";
    public $store = [];
    public $log = [];

    public function seed($key, $value) {
        $this->store[$key] = $value;
    }

    public function snapshotDynamicLocal() {
        $property = $this->which;
        $tmp = $this->{$property};
        $this->log[] = "dynamic";
        return $tmp;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$bag = new Milestone1881_Bag();
$bag->seed("x", "old");
$alias =& $bag["x"];
$copy = $bag->snapshotDynamicLocal();
$alias = "new";
$copy["x"] = "copy";
echo $bag->store["x"], "|", $alias, "|", $copy["x"], "|", implode(",", $bag->log);
