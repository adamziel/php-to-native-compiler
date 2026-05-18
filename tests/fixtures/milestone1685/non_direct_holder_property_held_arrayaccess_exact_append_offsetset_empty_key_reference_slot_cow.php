<?php
class Milestone1685_NonDirectHolderExactAppendStoredBucket implements ArrayAccess {
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

class Milestone1685_NonDirectHolderExactAppendHolder {
    public $bag;
}

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$holders = ["box" => new Milestone1685_NonDirectHolderExactAppendHolder()];
$holders["box"]->bag = new Milestone1685_NonDirectHolderExactAppendStoredBucket();
$holders["box"]->bag[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $holders["box"]->bag[null];
$bucket["id"]["function"] = "via-non-direct-empty-key";
$bucket["id"]["label"] = "via-non-direct-empty-label";
$bucket["id"]["accepted_args"] = 2;
$bucket["plain"]["function"] = "plain-copy";
$bucket["plain"]["label"] = "plain-copy";

$again = $holders["box"]->bag[null];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
