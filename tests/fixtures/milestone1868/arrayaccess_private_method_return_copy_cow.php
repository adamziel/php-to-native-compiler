<?php
class Bag implements ArrayAccess {
    private $store = [];
    public $log = [];

    public function seed($key, $value) {
        $this->store[$key] = $value;
    }

    public function read($key) {
        return $this->store[$key];
    }

    public function snapshot() {
        $this->log[] = "snapshot";
        return $this->store;
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
$copy = $bag->snapshot();
$alias = "new";
$copy["x"] = "copy";
echo $bag->read("x"), "|", $alias, "|", $copy["x"], "|", implode(",", $bag->log);
