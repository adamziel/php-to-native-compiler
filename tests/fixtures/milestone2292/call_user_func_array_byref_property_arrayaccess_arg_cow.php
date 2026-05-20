<?php
function pass_with_side_effect(&$sink, $arr) {
    $sink = "touched";
    return $arr;
}

class Bag implements ArrayAccess {
    public $store;
    public $sink;

    public function offsetGet(mixed $offset): mixed {
        $copy = $this->store[$offset];
        $this->store[$offset]["plain"] = "side";
        return $copy;
    }

    public function offsetSet(mixed $offset, mixed $value): void {
        $this->store[$offset] = $value;
    }

    public function offsetExists(mixed $offset): bool {
        return isset($this->store[$offset]);
    }

    public function offsetUnset(mixed $offset): void {
        unset($this->store[$offset]);
    }
}

$bag = new Bag();
$bag->store = array("a" => array("plain" => "old", "ref" => array("v" => "leaf")));
$alias =& $bag->store["a"]["ref"]["v"];
$args = array(&$bag->sink, $bag["a"]);
$tmp = call_user_func_array("pass_with_side_effect", $args);
$tmp["ref"]["v"] = "updated";
echo $bag->sink, "\n";
echo $bag->store["a"]["plain"], "\n";
echo $bag->store["a"]["ref"]["v"];
