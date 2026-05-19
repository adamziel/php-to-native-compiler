<?php
class Milestone1839_Hook implements ArrayAccess {
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

class Milestone1839_Holder {
    public $hook;
}

function milestone1839_mutate($bucket, $label) {
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
$holder = new Milestone1839_Holder();
$holder->hook = new Milestone1839_Hook();
$holder->hook->add(10, $callback);
$bucket = $holder->hook[10];
milestone1839_mutate($bucket, "held");

echo $callback, "|", $holder->hook->callbacks[10]["id"]["function"], "|", $holder->hook->callbacks[10]["plain"]["function"], "|", $holder->hook->hits[0];
