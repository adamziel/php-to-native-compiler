<?php
class Milestone1700_DetachLeafBag implements ArrayAccess {
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

class Milestone1700_DetachOuterBag implements ArrayAccess {
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

class Milestone1700_DetachBox {
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

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$leaf = new Milestone1700_DetachLeafBag();
$outer = new Milestone1700_DetachOuterBag($leaf);
$box = new Milestone1700_DetachBox($outer);
$alias =& $box->missing["outer"]["leaf"];
$alias[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];
unset($alias);
$alias = ["detached-local"];

$leaf->items["leaf"][0]["id"]["function"] = "via-unset-backing-source";
$leaf->items["leaf"][0]["id"]["label"] = "via-unset-backing-label";
$leaf->items["leaf"][0]["id"]["accepted_args"] = 4;
$leaf->items["leaf"][0]["plain"]["function"] = "plain-after-unset";
$leaf->items["leaf"][0]["plain"]["label"] = "plain-after-unset";

$again = $leaf->items["leaf"][0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"], "|", $alias[0], "|", count($leaf->items["leaf"]), "|", $box->gets, "|", $box->sets;
