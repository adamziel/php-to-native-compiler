<?php
class Milestone1890_Bag implements ArrayAccess {
    public $store = [];
    public $log = [];

    public function seed(&$leaf) {
        $this->store["outer"] = ["leaf" => &$leaf, "plain" => "old"];
    }

    private function choose($key) {
        if ($key === "outer") {
            return $this->store[$key];
        }
        return [];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        $bucket = $this->choose($offset);
        if ($offset === "outer") {
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

$leaf = "seed";
$bag = new Milestone1890_Bag();
$bag->seed($leaf);
$copy = $bag["outer"];
$copy["leaf"] = "copy";

echo $leaf, "|", $bag->store["outer"]["leaf"], "|", $copy["leaf"], "|", $bag->log[0];
