<?php
class Milestone1673_Control_Hook implements ArrayAccess {
    public $callbacks = [];

    public function add_one($priority, &$callback) {
        $this->callbacks[$priority] = [
            "id" => ["function" => &$callback, "accepted_args" => 1],
            "plain" => ["function" => "plain", "accepted_args" => 1],
        ];
    }

    public function add_two($priority, &$first, &$second) {
        $this->callbacks[$priority] = [
            "first" => ["function" => &$first, "accepted_args" => 1],
            "second" => ["function" => &$second, "accepted_args" => 1],
        ];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->callbacks[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->callbacks[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->callbacks[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->callbacks[$offset]);
    }
}

function milestone1673_control_two_refs($bucket) {
    foreach ($bucket as $id => &$callback) {
        $callback["function"] = $id . ":mutated";
    }
    unset($callback);
}

function milestone1673_control_report($label, $parts, $newline = true) {
    echo $label, ":", implode("|", $parts);
    if ($newline) {
        echo "\n";
    }
}

$callback = "seed";
$hook = new Milestone1673_Control_Hook();
$hook->add_one(10, $callback);
$bucket = $hook[10];
$bucket = ["id" => ["function" => "local", "accepted_args" => 9]];
$bucket["id"]["function"] = "bucket:reused";
milestone1673_control_report("outer-bucket-reuse", [
    $callback,
    $hook->callbacks[10]["id"]["function"],
    $bucket["id"]["function"],
]);

$callback = "seed";
$hook = new Milestone1673_Control_Hook();
$hook->add_one(10, $callback);
$bucket = $hook[10];
unset($callback);
$callback = "new-local";
foreach ($bucket as $id => &$loopCallback) {
    if ($id === "id") {
        $loopCallback["function"] = "after-callback-reuse";
    }
}
unset($loopCallback);
milestone1673_control_report("callback-name-reuse", [
    $callback,
    $hook->callbacks[10]["id"]["function"],
    $bucket["id"]["function"],
]);

$first = "a";
$second = "b";
$hook = new Milestone1673_Control_Hook();
$hook->add_two(10, $first, $second);
$bucket = $hook[10];
milestone1673_control_two_refs($bucket);
milestone1673_control_report("two-distinct-refs", [
    $first,
    $second,
    $hook->callbacks[10]["first"]["function"],
    $hook->callbacks[10]["second"]["function"],
], false);
