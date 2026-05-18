<?php
class Milestone1682_AppendStoredBucket implements ArrayAccess {
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

$target = "seed";
$node = ["function" => "placeholder", "accepted_args" => 1];
$node["function"] =& $target;

$bag = new Milestone1682_AppendStoredBucket();
$bag[] = [
    "id" => $node,
    "plain" => ["function" => "plain", "accepted_args" => 1],
];

$bucket = $bag[0];
foreach ($bucket as $id => &$callback) {
    if ($id === "id") {
        $callback["function"] = "via-append";
        $callback["accepted_args"] = 2;
    } else {
        $callback["function"] = "plain-copy";
    }
}
unset($callback);

$again = $bag[0];
echo $target, "|", $again["id"]["function"], "|", $again["id"]["accepted_args"], "|", $again["plain"]["function"];
