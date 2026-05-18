<?php
class Milestone1673_ParamReuse_Hook implements ArrayAccess {
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

function milestone1673_param_reuse($bucket) {
    foreach ($bucket as $id => &$callback) {
        if ($id === "id") {
            $callback["function"] = "param:first";
        }
    }
    unset($callback);

    $bucket = ["id" => ["function" => "local", "accepted_args" => 9]];
    $bucket["id"]["function"] = "param:reused";
}

$callback = "seed";
$hook = new Milestone1673_ParamReuse_Hook();
$hook->add(10, $callback);
$bucket = $hook[10];
milestone1673_param_reuse($bucket);
echo $callback, "|", $hook->callbacks[10]["id"]["function"], "|", $bucket["id"]["function"];
