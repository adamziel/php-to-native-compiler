<?php
class Milestone1877_Steps {
    public $first = "go";
}

class Milestone1877_Bag implements ArrayAccess {
    public $store = [];
    public $log = [];
    public $steps;

    public function seed($key, $value) {
        $this->store[$key] = $value;
        $this->steps = new Milestone1877_Steps();
    }

    public function snapshotForeachObject() {
        foreach ($this->steps as $name => $step) {
            $this->log[] = $name . ":" . $step;
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

$bag = new Milestone1877_Bag();
$bag->seed("x", "old");
$alias =& $bag["x"];
$copy = $bag->snapshotForeachObject();
$alias = "new";
$copy["x"] = "copy";
echo $bag->store["x"], "|", $alias, "|", $copy["x"], "|", implode(",", $bag->log);
