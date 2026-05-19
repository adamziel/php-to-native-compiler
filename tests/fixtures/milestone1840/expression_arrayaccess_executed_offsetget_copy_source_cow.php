<?php
class Milestone1840_Hook implements ArrayAccess {
    public $callbacks = array();
    public $hits = array();

    public function add($priority, &$callback) {
        $this->callbacks[$priority] = array(
            "id" => array("function" => &$callback, "accepted_args" => 1),
            "plain" => array("function" => "plain", "accepted_args" => 1),
        );
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->callbacks[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = "get:" . $offset;
        if (!isset($this->callbacks[$offset])) {
            $this->callbacks[$offset] = array();
        }
        $bucket =& $this->callbacks[$offset];
        return $bucket;
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

function milestone1840_make_hook(&$callback) {
    $hook = new Milestone1840_Hook();
    $hook->add(10, $callback);
    return $hook;
}

function milestone1840_mutate($bucket, $label) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = $label . ":ref";
        } else {
            $node["function"] = $label . ":plain";
        }
    }
    unset($node);
}

$callback = "seed";
$hook = milestone1840_make_hook($callback);
$bucket = $hook[10];
milestone1840_mutate($bucket, "expr");

echo $callback, "|", $hook->callbacks[10]["id"]["function"], "|", $hook->callbacks[10]["plain"]["function"], "|", $hook->hits[0];
