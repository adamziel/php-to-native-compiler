<?php
class Bag implements ArrayAccess {
    public $store = [];
    public $log = [];

    public function seed($key, $value) {
        $this->store[$key] = $value;
    }

    public function snapshotLoop() {
        $this->log[] = "loop";
        for ($i = 0; $i < 1; $i++) {
            return $this->store;
        }
        return ["x" => "fallback"];
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
$copy = $bag->snapshotLoop();
$alias = "new";
$copy["x"] = "copy";
echo $bag->store["x"], "|", $alias, "|", $copy["x"], "|", implode(",", $bag->log);
