<?php
class Milestone1667PublicBag {
    public $first = "one";
    public $second = "two";
    public $dynamic = "dyn";
    private $hidden = "secret";
}

$bag = new Milestone1667PublicBag();
foreach ($bag as $key => $value) {
    echo $key, "=", $value, ";";
    if ($key === "first") {
        $bag->second = "mutated";
    }
}
echo "|", $bag->second, "\n";

class Milestone1667Iterator implements Iterator {
    public $items = array("first" => "alpha", "second" => "beta");
    public $keys = array("first", "second");
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
    public function current() {
        $key = $this->keys[$this->pos];
        return $this->items[$key];
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->keys[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }
}

$iterator = new Milestone1667Iterator();
foreach ($iterator as $key => $value) {
    echo $key, "=", $value, ";";
    if ($key === "first") {
        $iterator->items["second"] = "changed";
    }
}
echo "|", $iterator->pos;
