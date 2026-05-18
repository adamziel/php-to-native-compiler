<?php
class Milestone1673_Stored_Cufa_Hook implements ArrayAccess {
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

class Milestone1673_Stored_Cufa_Holder {
    public $hook;
}

function milestone1673_make_hook(&$callback) {
    $hook = new Milestone1673_Stored_Cufa_Hook();
    $hook->add(10, $callback);
    return $hook;
}

function milestone1673_make_holder($hook) {
    $holder = new Milestone1673_Stored_Cufa_Holder();
    $holder->hook = $hook;
    return $holder;
}

function milestone1673_mutate_bucket($bucket, $label) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = $label . ":ref";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = $label . ":plain";
        }
    }
    unset($node);
}

$callback = "seed";
$hook = milestone1673_make_hook($callback);
$bucket = $hook[10];
$args = [$bucket, "direct"];
call_user_func_array("milestone1673_mutate_bucket", $args);
echo $callback, "|", $hook->callbacks[10]["id"]["function"], "|", $hook->callbacks[10]["id"]["accepted_args"], "|", $hook->callbacks[10]["plain"]["function"], "\n";

$propertyCallback = "seed";
$propertyHook = milestone1673_make_hook($propertyCallback);
$holder = milestone1673_make_holder($propertyHook);
$propertyBucket = $holder->hook[10];
$propertyArgs = [$propertyBucket, "property"];
call_user_func_array("milestone1673_mutate_bucket", $propertyArgs);
echo $propertyCallback, "|", $propertyHook->callbacks[10]["id"]["function"], "|", $propertyHook->callbacks[10]["id"]["accepted_args"], "|", $propertyHook->callbacks[10]["plain"]["function"];
