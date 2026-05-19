<?php
class Milestone1883_Bag implements ArrayAccess {
    public $store = [];
    public $log = [];

    public function seed($group, $key, $value) {
        $this->store[$group][$key] = $value;
    }

    public function snapshotOffsetLocal($group) {
        $tmp = $this[$group];
        $this->log[] = "offset:" . $group;
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

$bag = new Milestone1883_Bag();
$bag->seed("outer", "leaf", "old");
$alias =& $bag["outer"]["leaf"];
$copy = $bag->snapshotOffsetLocal("outer");
$alias = "new";
$copy["leaf"] = "copy";
echo $bag->store["outer"]["leaf"], "|", $alias, "|", $copy["leaf"], "|", implode(",", $bag->log);
