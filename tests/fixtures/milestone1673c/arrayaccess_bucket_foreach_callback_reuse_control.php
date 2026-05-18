<?php
class Milestone1673_CallbackReuse_Hook implements ArrayAccess {
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

function milestone1673_callback_reuse($bucket) {
    foreach ($bucket as $id => &$callback) {
        if ($id === "id") {
            $callback["function"] = "loop:first";
        }
    }

    $callback = ["function" => "loop:reused", "accepted_args" => 9];
}

$callback = "seed";
$hook = new Milestone1673_CallbackReuse_Hook();
$hook->add(10, $callback);
$bucket = $hook[10];
milestone1673_callback_reuse($bucket);
echo $callback, "|", $hook->callbacks[10]["id"]["function"], "|", $bucket["id"]["function"];
