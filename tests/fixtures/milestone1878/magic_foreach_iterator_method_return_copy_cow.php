<?php
class Milestone1878_StepsIterator implements Iterator {
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

class Milestone1878_Box {
    private $store = [];
    public $log = [];
    public $steps;

    public function seed($key, $value) {
        $this->store[$key] = $value;
        $this->steps = new Milestone1878_StepsIterator();
    }

    public function read($key) {
        return $this->store[$key];
    }

    public function &__get($name) {
        $this->log[] = "get:" . $name;
        return $this->store[$name];
    }

    public function snapshotForeachIterator() {
        foreach ($this->steps as $name => $step) {
            $this->log[] = $name . ":" . $step;
            return $this->store;
        }
        return ["x" => "fallback"];
    }
}

$box = new Milestone1878_Box();
$box->seed("x", "old");
$alias =& $box->x;
$copy = $box->snapshotForeachIterator();
$alias = "new";
$copy["x"] = "copy";
echo $box->read("x"), "|", $alias, "|", $copy["x"], "|", implode(",", $box->log);
