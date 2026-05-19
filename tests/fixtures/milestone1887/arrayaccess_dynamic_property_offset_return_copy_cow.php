<?php
class Milestone1887_Bag implements ArrayAccess {
    public $store = [];

    public function seed($group, $key, $value) {
        $this->store[$group][$key] = $value;
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

class Milestone1887_Holder {
    public $which = "bag";
    public $bag;
    public $log = [];

    public function snapshotDynamicProperty($group) {
        $property = $this->which;
        $this->log[] = "dynamic:" . $group;
        return $this->{$property}[$group];
    }
}

$bag = new Milestone1887_Bag();
$bag->seed("outer", "leaf", "old");
$alias =& $bag["outer"]["leaf"];
$holder = new Milestone1887_Holder();
$holder->bag = $bag;
$copy = $holder->snapshotDynamicProperty("outer");
$alias = "new";
$copy["leaf"] = "copy";
echo $bag->store["outer"]["leaf"], "|", $alias, "|", $copy["leaf"], "|", implode(",", $holder->log);
