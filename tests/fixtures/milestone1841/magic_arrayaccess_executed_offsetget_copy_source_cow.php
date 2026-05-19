<?php
class Milestone1841_Hook implements ArrayAccess {
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

class Milestone1841_Box {
    private $hook;

    public function __construct(&$callback) {
        $this->hook = new Milestone1841_Hook();
        $this->hook->add(10, $callback);
    }

    public function __get($name) {
        return $this->hook;
    }

    public function hit($index) {
        return $this->hook->hits[$index];
    }
}

function milestone1841_mutate($bucket, $label) {
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
$box = new Milestone1841_Box($callback);
$bucket = $box->missing[10];
milestone1841_mutate($bucket, "magic");

echo $callback, "|", $box->hit(0);
