<?php
class Milestone1696_ByValueOuterReferenceInnerBag implements ArrayAccess {
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

class Milestone1696_ByValueOuterReferenceBag implements ArrayAccess {
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

class Milestone1696_ByValueOuterReferenceBox {
    public function &__get($name) {
        global $milestone1696_outer_store;
        return $milestone1696_outer_store;
    }

    public function __set($name, $value) {
    }
}

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder", "accepted_args" => 1];
$node["function"] =& $function;
$node["label"] =& $label;

$inner = new Milestone1696_ByValueOuterReferenceInnerBag();
$outer = new Milestone1696_ByValueOuterReferenceBag($inner);
$milestone1696_outer_store = $outer;
$box = new Milestone1696_ByValueOuterReferenceBox();
$alias =& $box->missing["outer"]["inner"];
$alias[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "label" => "plain", "accepted_args" => 1],
];

$bucket = $box->missing["outer"]["inner"][0];
foreach ($bucket as $id => &$callback) {
    if ($id === "id") {
        $callback["function"] = "via-by-value-outer-reference-source";
        $callback["label"] = "via-by-value-outer-reference-label";
        $callback["accepted_args"] = 2;
    } else {
        $callback["function"] = "plain-copy";
        $callback["label"] = "plain-copy";
    }
}
unset($callback);

$again = $inner->items["inner"][0];
echo $function, "|", $label, "|", $again["id"]["function"], "|", $again["id"]["label"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"], "|", $again["plain"]["label"];
