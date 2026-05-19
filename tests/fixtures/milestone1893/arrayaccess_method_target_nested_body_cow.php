<?php
class Milestone1893_Bag implements ArrayAccess {
    public $store = [];
    public $log = [];

    public function seed(&$leaf) {
        $this->store["outer"] = [
            "group" => ["leaf" => &$leaf, "plain" => "old"],
        ];
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
        $selected = $this->bucket($offset)["group"];
        return $selected;
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
$bag = new Milestone1893_Bag();
$bag->seed($leaf);
$copy = $bag["outer"];
$copy["leaf"] = "copy";

echo $leaf, "|", $bag->store["outer"]["group"]["leaf"], "|", $copy["leaf"], "|", $bag->log[0];
