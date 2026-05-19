<?php
class Milestone1888_Bag implements ArrayAccess {
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

class Milestone1888_Holder {
    public $bag;
    public $log = [];

    public function make() {
        $this->log[] = "make";
        return $this->bag;
    }

    public function snapshotExpression($group) {
        return $this->make()[$group];
    }
}

$bag = new Milestone1888_Bag();
$bag->seed("outer", "leaf", "old");
$alias =& $bag["outer"]["leaf"];
$holder = new Milestone1888_Holder();
$holder->bag = $bag;
$copy = $holder->snapshotExpression("outer");
$alias = "new";
$copy["leaf"] = "copy";
echo $bag->store["outer"]["leaf"], "|", $alias, "|", $copy["leaf"], "|", implode(",", $holder->log);
