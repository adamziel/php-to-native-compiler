<?php
class Milestone1894_Bag implements ArrayAccess {
    public $store = [];
    public $log = [];

    public function seed(&$leaf) {
        $this->store["group"] = ["leaf" => &$leaf, "plain" => "old"];
    }

    private function bucket($key) {
        $this->log[] = "bucket:" . $key;
        return $this->store[$key];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = $this->bucket($offset);
        if ($offset === "group") {
            return $bucket;
        }
        return [];
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

class Milestone1894_Box {
    public $bags = [];
    public $log = [];

    public function __construct($bag) {
        $this->bags["outer"] = $bag;
    }

    private function bag($name) {
        return $this->bags[$name];
    }

    public function __get($name) {
        $this->log[] = "get:" . $name;
        $bag = $this->bag($name);
        return $bag;
    }
}

$leaf = "seed";
$bag = new Milestone1894_Bag();
$bag->seed($leaf);
$box = new Milestone1894_Box($bag);
$copy = $box->outer["group"];
$copy["leaf"] = "copy";

echo $leaf, "|", $bag->store["group"]["leaf"], "|", $copy["leaf"], "|", $box->log[0], "|", $bag->log[0];
