<?php
class Milestone1697_ByValueMagicReferenceInnerBag implements ArrayAccess {
    public $items = ["inner" => []];

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

class Milestone1697_ByValueMagicReferenceOuterBag implements ArrayAccess {
    public $items = [];

    public function __construct($inner) {
        $this->items["outer"] = $inner;
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

class Milestone1697_ByValueMagicReferenceBox {
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

$inner = new Milestone1697_ByValueMagicReferenceInnerBag();
$outer = new Milestone1697_ByValueMagicReferenceOuterBag($inner);
$box = new Milestone1697_ByValueMagicReferenceBox($outer);
$alias =& $box->missing["outer"]["inner"];
$alias[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $box->missing["outer"]["inner"][0];
foreach ($bucket as $id => &$callback) {
    if ($id === "id") {
        $callback["function"] = "via-by-value-magic-reference-source";
        $callback["label"] = "via-by-value-magic-reference-label";
        $callback["accepted_args"] = 2;
    } else {
        $callback["function"] = "plain-copy";
        $callback["label"] = "plain-copy";
    }
}
unset($callback);

$again = $inner->items["inner"][0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"], "|", $box->gets, "|", $box->sets;
