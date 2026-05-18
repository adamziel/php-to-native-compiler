<?php
class Milestone1663_ArrayAccess_Bag implements ArrayAccess {
    private $storage = ["slot" => "seed"];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->storage[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->storage[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->storage[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->storage[$offset]);
    }

    public function read($offset) {
        return $this->storage[$offset];
    }
}

$bag = new Milestone1663_ArrayAccess_Bag();

class Milestone1663_Magic_Box {
    public function &__get($name) {
        global $bag;
        return $bag;
    }
}

function milestone1663_touch(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
}

$box = new Milestone1663_Magic_Box();
milestone1663_touch($box->missing["slot"], "arg");
echo $bag->read("slot"), "\n";

$alias =& $box->missing["created"];
$alias = "via-alias";
echo $bag->read("created"), "|", $alias;
