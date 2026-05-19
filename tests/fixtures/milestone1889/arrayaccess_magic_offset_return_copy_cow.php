<?php
class Milestone1889_Bag implements ArrayAccess {
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

class Milestone1889_Box {
    public $bag;
    public $log = [];

    public function __get($name) {
        $this->log[] = "get:" . $name;
        return $this->bag;
    }

    public function snapshotMagic($group) {
        return $this->missing[$group];
    }
}

$bag = new Milestone1889_Bag();
$bag->seed("outer", "leaf", "old");
$alias =& $bag["outer"]["leaf"];
$box = new Milestone1889_Box();
$box->bag = $bag;
$copy = $box->snapshotMagic("outer");
$alias = "new";
$copy["leaf"] = "copy";
echo $bag->store["outer"]["leaf"], "|", $alias, "|", $copy["leaf"], "|", implode(",", $box->log);
