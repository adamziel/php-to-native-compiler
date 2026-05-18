<?php
class Milestone1688_NonDirectMagicExactAppendStoredBucket implements ArrayAccess {
    private $items = [];

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

class Milestone1688_NonDirectMagicExactAppendBox {
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

$holders = ["box" => new Milestone1688_NonDirectMagicExactAppendBox(new Milestone1688_NonDirectMagicExactAppendStoredBucket())];
$holders["box"]->missing[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];
$store_gets = $holders["box"]->gets;
$store_sets = $holders["box"]->sets;

$bucket = $holders["box"]->missing[null];
$bucket["id"]["function"] = "via-non-direct-magic-empty-key";
$bucket["id"]["label"] = "via-non-direct-magic-empty-label";
$bucket["id"]["accepted_args"] = 2;
$bucket["plain"]["function"] = "plain-copy";
$bucket["plain"]["label"] = "plain-copy";

$again = $holders["box"]->missing[null];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"], "|", $store_gets, "|", $store_sets;
