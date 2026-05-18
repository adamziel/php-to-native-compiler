<?php
class Milestone1670_Value_ArrayAccess_Hook implements ArrayAccess {
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

class Milestone1670_Ref_ArrayAccess_Hook implements ArrayAccess {
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
    public function &offsetGet($offset) {
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

function milestone1670_exercise_bucket_copy($hook, $label) {
    $bucket = $hook[10];
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = $label . ":copy";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = $label . ":plain-copy";
        }
    }
    unset($node);
}

$callback = "seed";
$valueHook = new Milestone1670_Value_ArrayAccess_Hook();
$valueHook->add(10, $callback);
milestone1670_exercise_bucket_copy($valueHook, "value");
echo $callback, "|", $valueHook->callbacks[10]["id"]["function"], "|", $valueHook->callbacks[10]["id"]["accepted_args"], "|", $valueHook->callbacks[10]["plain"]["function"], "\n";

$refCallback = "seed";
$refHook = new Milestone1670_Ref_ArrayAccess_Hook();
$refHook->add(10, $refCallback);
milestone1670_exercise_bucket_copy($refHook, "ref");
echo $refCallback, "|", $refHook->callbacks[10]["id"]["function"], "|", $refHook->callbacks[10]["id"]["accepted_args"], "|", $refHook->callbacks[10]["plain"]["function"];
