<?php
class Milestone1671_ArrayAccess_Hook implements ArrayAccess {
    public $callbacks = [];

    public function add($priority, &$callback) {
        $this->callbacks[$priority] = [
            "id" => ["function" => &$callback, "accepted_args" => 1],
            "plain" => ["function" => "plain", "accepted_args" => 1],
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

class Milestone1671_Holder {
    public $hook;
    public $dynamicHook;
}

function milestone1671_make_holder($hook) {
    $holder = new Milestone1671_Holder();
    $holder->hook = $hook;
    return $holder;
}

$holder = new Milestone1671_Holder();

$callback = "seed";
$holder->hook = new Milestone1671_ArrayAccess_Hook();
$holder->hook->add(10, $callback);
$bucket = $holder->hook[10];
foreach ($bucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "holder:copy";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "holder:plain-copy";
    }
}
unset($node);
echo $callback, "|", $holder->hook->callbacks[10]["id"]["function"], "|", $holder->hook->callbacks[10]["id"]["accepted_args"], "|", $holder->hook->callbacks[10]["plain"]["function"], "\n";

$dynamicCallback = "seed";
$holder->dynamicHook = new Milestone1671_ArrayAccess_Hook();
$holder->dynamicHook->add(10, $dynamicCallback);
$property = "dynamicHook";
$dynamicBucket = $holder->{$property}[10];
foreach ($dynamicBucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "dynamic:copy";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "dynamic:plain-copy";
    }
}
unset($node);
echo $dynamicCallback, "|", $holder->dynamicHook->callbacks[10]["id"]["function"], "|", $holder->dynamicHook->callbacks[10]["id"]["accepted_args"], "|", $holder->dynamicHook->callbacks[10]["plain"]["function"];

$exprCallback = "seed";
$exprHook = new Milestone1671_ArrayAccess_Hook();
$exprHook->add(10, $exprCallback);
$exprBucket = milestone1671_make_holder($exprHook)->hook[10];
foreach ($exprBucket as $id => &$node) {
    if ($id === "id") {
        $node["function"] = "expr:copy";
        $node["accepted_args"] = 2;
    } else {
        $node["function"] = "expr:plain-copy";
    }
}
unset($node);
echo "\n", $exprCallback, "|", $exprHook->callbacks[10]["id"]["function"], "|", $exprHook->callbacks[10]["id"]["accepted_args"], "|", $exprHook->callbacks[10]["plain"]["function"];
