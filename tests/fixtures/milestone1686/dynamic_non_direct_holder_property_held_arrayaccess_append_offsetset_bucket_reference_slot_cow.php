<?php
class Milestone1686_DynamicNonDirectHolderAppendStoredBucket implements ArrayAccess {
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
        if ($offset === null) {
            $this->items[] = $value;
            return;
        }

        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1686_DynamicNonDirectHolderAppendHolder {
    public $bag;
}

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$name = "bag";
$holders = ["box" => new Milestone1686_DynamicNonDirectHolderAppendHolder()];
$holders["box"]->bag = new Milestone1686_DynamicNonDirectHolderAppendStoredBucket();
$holders["box"]->{$name}[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $holders["box"]->{$name}[0];
foreach ($bucket as $id => &$callback) {
    if ($id === "id") {
        $callback["function"] = "via-dynamic-non-direct-append";
        $callback["label"] = "via-dynamic-non-direct-label";
        $callback["accepted_args"] = 2;
    } else {
        $callback["function"] = "plain-copy";
        $callback["label"] = "plain-copy";
    }
}
unset($callback);

$again = $holders["box"]->{$name}[0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
