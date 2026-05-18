<?php
class Milestone1699_FactoryLeafBag implements ArrayAccess {
    public $items = ["leaf" => []];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1699_FactoryOuterBag implements ArrayAccess {
    public $items = [];

    public function __construct($leaf) {
        $this->items["outer"] = $leaf;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1699_FactoryBox {
    public $gets = 0;
    public $sets = 0;
    private $store;

    public function __construct($store) {
        $this->store = $store;
    }

    public function __get($name) {
        $this->gets = $this->gets + 1;
        return $this->store;
    }

    public function __set($name, $value) {
        $this->sets = $this->sets + 1;
    }
}

function milestone1699_factory_box() {
    global $box;
    return $box;
}

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$leaf = new Milestone1699_FactoryLeafBag();
$outer = new Milestone1699_FactoryOuterBag($leaf);
$box = new Milestone1699_FactoryBox($outer);
$alias =& milestone1699_factory_box()->missing["outer"]["leaf"];
$alias[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$leaf->items["leaf"][0]["id"]["function"] = "via-factory-backing-source";
$leaf->items["leaf"][0]["id"]["label"] = "via-factory-backing-label";
$leaf->items["leaf"][0]["id"]["accepted_args"] = 3;
$leaf->items["leaf"][0]["plain"]["function"] = "plain-direct";
$leaf->items["leaf"][0]["plain"]["label"] = "plain-direct";

$again = $leaf->items["leaf"][0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"], "|", $box->gets, "|", $box->sets;
