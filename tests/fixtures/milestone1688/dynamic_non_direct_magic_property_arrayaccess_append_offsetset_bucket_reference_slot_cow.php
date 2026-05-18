<?php
class Milestone1688_DynamicNonDirectMagicAppendStoredBucket implements ArrayAccess {
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

class Milestone1688_DynamicNonDirectMagicAppendBox {
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

$name = "missing";
$holders = ["box" => new Milestone1688_DynamicNonDirectMagicAppendBox(new Milestone1688_DynamicNonDirectMagicAppendStoredBucket())];
$holders["box"]->{$name}[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];
$store_gets = $holders["box"]->gets;
$store_sets = $holders["box"]->sets;

$bucket = $holders["box"]->{$name}[0];
foreach ($bucket as $id => &$callback) {
    if ($id === "id") {
        $callback["function"] = "via-dynamic-non-direct-magic-append";
        $callback["label"] = "via-dynamic-non-direct-magic-label";
        $callback["accepted_args"] = 2;
    } else {
        $callback["function"] = "plain-copy";
        $callback["label"] = "plain-copy";
    }
}
unset($callback);

$again = $holders["box"]->{$name}[0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"], "|", $store_gets, "|", $store_sets;
