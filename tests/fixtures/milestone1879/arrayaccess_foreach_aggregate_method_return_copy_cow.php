<?php
class Milestone1879_StepsIterator implements Iterator {
    public $items = ["first" => "go"];
    public $keys = ["first"];
    public $pos = 0;

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->pos = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return isset($this->keys[$this->pos]);
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->keys[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function current() {
        return $this->items[$this->keys[$this->pos]];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }
}

class Milestone1879_StepsAggregate implements IteratorAggregate {
    public $iterator;

    public function __construct() {
        $this->iterator = new Milestone1879_StepsIterator();
    }

    #[ReturnTypeWillChange]
    public function getIterator() {
        return $this->iterator;
    }
}

class Milestone1879_Bag implements ArrayAccess {
    public $store = [];
    public $log = [];
    public $steps;

    public function seed($key, $value) {
        $this->store[$key] = $value;
        $this->steps = new Milestone1879_StepsAggregate();
    }

    public function snapshotForeachAggregate() {
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

$bag = new Milestone1879_Bag();
$bag->seed("x", "old");
$alias =& $bag["x"];
$copy = $bag->snapshotForeachAggregate();
$alias = "new";
$copy["x"] = "copy";
echo $bag->store["x"], "|", $alias, "|", $copy["x"], "|", implode(",", $bag->log);
