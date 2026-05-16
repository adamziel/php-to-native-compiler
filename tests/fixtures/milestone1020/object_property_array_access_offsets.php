<?php
class Bag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        echo "exists:$offset\n";
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        echo "get:$offset\n";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        echo "set:$offset:$value\n";
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        echo "unset:$offset\n";
        unset($this->items[$offset]);
    }
}

class Holder {
    public $bag;
}

$holder = new Holder();
$holder->bag = new Bag();
$holder->bag["name"] = "Ada";
echo $holder->bag["name"], "\n";
echo isset($holder->bag["name"]) ? "isset\n" : "missing\n";
echo empty($holder->bag["name"]) ? "empty\n" : "not-empty\n";
echo $holder->bag["missing"] ?? "fallback", "\n";
unset($holder->bag["name"]);
echo isset($holder->bag["name"]) ? "isset" : "missing";
