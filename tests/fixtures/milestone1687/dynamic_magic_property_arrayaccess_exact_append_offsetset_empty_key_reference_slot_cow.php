<?php
class Milestone1687_DynamicMagicExactAppendStoredBucket implements ArrayAccess {
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

class Milestone1687_DynamicMagicExactAppendBox {
    private $store;

    public function __construct($store) {
        $this->store = $store;
    }

    public function __get($name) {
        return $this->store;
    }
}

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$name = "missing";
$box = new Milestone1687_DynamicMagicExactAppendBox(new Milestone1687_DynamicMagicExactAppendStoredBucket());
$box->{$name}[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $box->{$name}[null];
$bucket["id"]["function"] = "via-dynamic-magic-empty-key";
$bucket["id"]["label"] = "via-dynamic-magic-empty-label";
$bucket["id"]["accepted_args"] = 2;
$bucket["plain"]["function"] = "plain-copy";
$bucket["plain"]["label"] = "plain-copy";

$again = $box->{$name}[null];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
