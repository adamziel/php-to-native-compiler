<?php
class Bag implements ArrayAccess {
    public $which = "store";
    public $store = [];
    public $log = [];

    public function seed($key, $value) {
        $this->store[$key] = $value;
    }

    public function snapshotDynamic() {
        $property = $this->which;
        $this->log[] = "dynamic";
        return $this->{$property};
    }

    public function offsetExists($offset): bool {
        return true;
    }

    public function &offsetGet($offset): mixed {
        return $this->store[$offset];
    }

    public function offsetSet($offset, $value): void {
        $this->store[$offset] = $value;
    }

    public function offsetUnset($offset): void {
        unset($this->store[$offset]);
    }
}

$bag = new Bag();
$bag->seed("x", "old");
$alias =& $bag["x"];
$copy = $bag->snapshotDynamic();
$alias = "new";
$copy["x"] = "copy";
echo $bag->store["x"], "|", $alias, "|", $copy["x"], "|", implode(",", $bag->log);
