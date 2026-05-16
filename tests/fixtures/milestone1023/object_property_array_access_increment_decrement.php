<?php
error_reporting(0);

class Bag implements ArrayAccess {
    public $items = ["n" => 2, "f" => 1.5];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
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
        unset($this->items[$offset]);
    }
}

class Holder {
    public $bag;
}

$holder = new Holder();
$holder->bag = new Bag();
echo $holder->bag["n"]++, "\n";
echo ++$holder->bag["n"], "\n";
echo $holder->bag["f"]--, "\n";
echo --$holder->bag["f"], "\n";
$holder->bag["n"]++;
echo $holder->bag["n"];
