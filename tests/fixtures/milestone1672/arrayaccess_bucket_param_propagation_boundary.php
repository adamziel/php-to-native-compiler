<?php
class Milestone1672_Param_Hook implements ArrayAccess {
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

function milestone1672_make_hook(&$callback) {
    $hook = new Milestone1672_Param_Hook();
    $hook->add(10, $callback);
    return $hook;
}

function milestone1672_mutate_bucket($bucket, $label) {
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

function milestone1672_report($label, $callback, $hook, $newline = true) {
    echo $label, ":", $callback, "|", $hook->callbacks[10]["id"]["function"], "|", $hook->callbacks[10]["id"]["accepted_args"], "|", $hook->callbacks[10]["plain"]["function"];
    if ($newline) {
        echo "\n";
    }
}

$directCallback = "seed";
$directHook = milestone1672_make_hook($directCallback);
$directBucket = $directHook[10];
milestone1672_mutate_bucket($directBucket, "direct-fn");
milestone1672_report("direct-fn", $directCallback, $directHook);

$closureCallback = "seed";
$closureHook = milestone1672_make_hook($closureCallback);
$closure = function($bucket, $label) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = $label . ":ref";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = $label . ":plain";
        }
    }
    unset($node);
};
$closureBucket = $closureHook[10];
$closure($closureBucket, "closure");
milestone1672_report("closure", $closureCallback, $closureHook);

$cufCallback = "seed";
$cufHook = milestone1672_make_hook($cufCallback);
$cufBucket = $cufHook[10];
call_user_func("milestone1672_mutate_bucket", $cufBucket, "cuf");
milestone1672_report("cuf", $cufCallback, $cufHook);

$cufaCallback = "seed";
$cufaHook = milestone1672_make_hook($cufaCallback);
$cufaBucket = $cufaHook[10];
call_user_func_array("milestone1672_mutate_bucket", [$cufaBucket, "cufa"]);
milestone1672_report("cufa", $cufaCallback, $cufaHook, false);
