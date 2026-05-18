<?php
class Milestone1684_DynamicPropertyHeldExactAppendStoredBucket implements ArrayAccess {
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

class Milestone1684_DynamicPropertyHeldExactAppendHolder {
    public $bag;
}

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$holder = new Milestone1684_DynamicPropertyHeldExactAppendHolder();
$name = "bag";
$holder->{$name} = new Milestone1684_DynamicPropertyHeldExactAppendStoredBucket();
$holder->{$name}[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $holder->{$name}[null];
$bucket["id"]["function"] = "via-dynamic-empty-key";
$bucket["id"]["label"] = "via-dynamic-empty-label";
$bucket["id"]["accepted_args"] = 2;
$bucket["plain"]["function"] = "plain-copy";
$bucket["plain"]["label"] = "plain-copy";

$again = $holder->{$name}[null];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
