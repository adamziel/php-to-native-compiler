<?php
class Bag implements ArrayAccess {
    public $store = [];
    public $log = [];

    public function seed($group, $key, $value) {
        $this->store[$group][$key] = $value;
    }

    public function snapshotGroup($group) {
        $this->log[] = "group:" . $group;
        return $this->store[$group];
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
$bag->seed("outer", "leaf", "old");
$alias =& $bag["outer"]["leaf"];
$copy = $bag->snapshotGroup("outer");
$alias = "new";
$copy["leaf"] = "copy";
echo $bag->store["outer"]["leaf"], "|", $alias, "|", $copy["leaf"], "|", implode(",", $bag->log);
