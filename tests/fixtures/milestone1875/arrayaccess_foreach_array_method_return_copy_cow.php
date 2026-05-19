<?php
class Milestone1875_Bag implements ArrayAccess {
    public $store = [];
    public $log = [];

    public function seed($key, $value) {
        $this->store[$key] = $value;
    }

    public function snapshotForeachArray() {
        foreach (["go"] as $step) {
            $this->log[] = "array:" . $step;
            return $this->store;
        }
        return ["x" => "fallback"];
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

$bag = new Milestone1875_Bag();
$bag->seed("x", "old");
$alias =& $bag["x"];
$copy = $bag->snapshotForeachArray();
$alias = "new";
$copy["x"] = "copy";
echo $bag->store["x"], "|", $alias, "|", $copy["x"], "|", implode(",", $bag->log);
