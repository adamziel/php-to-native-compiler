<?php
class Milestone1672_ArrayAccess_Hook implements ArrayAccess {
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

class Milestone1672_Holder {
    public $hook;
}

function milestone1672_make_holder($hook) {
    $holder = new Milestone1672_Holder();
    $holder->hook = $hook;
    return $holder;
}

function milestone1672_helper($bucket, $label) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = $label . ":helper";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = $label . ":plain-helper";
        }
    }
    unset($node);
}

$callback = "seed";
$holder = new Milestone1672_Holder();
$holder->hook = new Milestone1672_ArrayAccess_Hook();
$holder->hook->add(10, $callback);
$bucket = $holder->hook[10];
milestone1672_helper($bucket, "property");
echo $callback, "|", $holder->hook->callbacks[10]["id"]["function"], "|", $holder->hook->callbacks[10]["id"]["accepted_args"], "|", $holder->hook->callbacks[10]["plain"]["function"], "\n";

$arrayCallback = "seed";
$arrayHook = new Milestone1672_ArrayAccess_Hook();
$arrayHook->add(10, $arrayCallback);
$holders = ["hook" => milestone1672_make_holder($arrayHook)];
$arrayBucket = $holders["hook"]->hook[10];
milestone1672_helper($arrayBucket, "array-holder");
echo $arrayCallback, "|", $arrayHook->callbacks[10]["id"]["function"], "|", $arrayHook->callbacks[10]["id"]["accepted_args"], "|", $arrayHook->callbacks[10]["plain"]["function"], "\n";

$exprCallback = "seed";
$exprHook = new Milestone1672_ArrayAccess_Hook();
$exprHook->add(10, $exprCallback);
$exprBucket = milestone1672_make_holder($exprHook)->hook[10];
milestone1672_helper($exprBucket, "expr-holder");
echo $exprCallback, "|", $exprHook->callbacks[10]["id"]["function"], "|", $exprHook->callbacks[10]["id"]["accepted_args"], "|", $exprHook->callbacks[10]["plain"]["function"];
