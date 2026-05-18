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

function milestone1673_helper($bucket, $label) {
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

function milestone1673_report($label, $callback, $hook, $args, $newline = true) {
    echo $label, ":", $callback, "|", $hook->callbacks[10]["id"]["function"], "|", $hook->callbacks[10]["id"]["accepted_args"], "|", $hook->callbacks[10]["plain"]["function"], "|", $args[0]["id"]["function"], "|", $args[0]["id"]["accepted_args"], "|", $args[0]["plain"]["function"];
    if ($newline) {
        echo "\n";
    }
}

$directCallback = "seed";
$directHook = milestone1673_make_hook($directCallback);
$directBucket = $directHook[10];
$directArgs = [$directBucket, "direct"];
call_user_func_array("milestone1673_helper", $directArgs);
milestone1673_report("direct", $directCallback, $directHook, $directArgs);

$propertyCallback = "seed";
$propertyHook = milestone1673_make_hook($propertyCallback);
$propertyHolder = milestone1673_make_holder($propertyHook);
$propertyBucket = $propertyHolder->hook[10];
$propertyArgs = [$propertyBucket, "property"];
call_user_func_array("milestone1673_helper", $propertyArgs);
milestone1673_report("property", $propertyCallback, $propertyHook, $propertyArgs);

$arrayHolderCallback = "seed";
$arrayHolderHook = milestone1673_make_hook($arrayHolderCallback);
$holders = ["box" => milestone1673_make_holder($arrayHolderHook)];
$arrayHolderBucket = $holders["box"]->hook[10];
$arrayHolderArgs = [$arrayHolderBucket, "array-holder"];
call_user_func_array("milestone1673_helper", $arrayHolderArgs);
milestone1673_report("array-holder", $arrayHolderCallback, $arrayHolderHook, $arrayHolderArgs);
