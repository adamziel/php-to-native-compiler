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
        echo "set:" . ($offset === null ? "null" : $offset) . ":$value\n";
        if ($offset === null) {
            $this->items[] = $value;
        } else {
            $this->items[$offset] = $value;
        }
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        echo "unset:$offset\n";
        unset($this->items[$offset]);
    }
}

$bag = new Bag();
$bag["name"] = "Ada";
echo $bag["name"], "\n";
echo isset($bag["name"]) ? "isset\n" : "missing\n";
echo empty($bag["name"]) ? "empty\n" : "not-empty\n";
echo $bag["missing"] ?? "fallback", "\n";
unset($bag["name"]);
echo isset($bag["name"]) ? "isset\n" : "missing\n";
$bag[] = "tail";
echo $bag[0];
