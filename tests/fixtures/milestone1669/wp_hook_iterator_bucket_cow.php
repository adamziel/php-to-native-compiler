<?php
class Milestone1669Hook implements Iterator {
    public $callbacks = array();
    public $priorities = array();
    public $pos = 0;

    public function add($priority, &$callback) {
        $this->callbacks[$priority] = array(
            "id" => array("function" => &$callback, "accepted_args" => 1),
            "plain" => array("function" => "plain", "accepted_args" => 1),
        );
        $this->priorities[] = $priority;
    }

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->pos = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return isset($this->priorities[$this->pos]);
    }

    #[ReturnTypeWillChange]
    public function current() {
        $priority = $this->priorities[$this->pos];
        return $this->callbacks[$priority];
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->priorities[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }
}

$callback = "seed";
$hook = new Milestone1669Hook();
$hook->add(10, $callback);

foreach ($hook as $priority => $bucket) {
    foreach ($bucket as $id => &$node) {
        if ($id === "id") {
            $node["function"] = "via-copy";
            $node["accepted_args"] = 2;
        } else {
            $node["function"] = "plain-copy";
        }
    }
    unset($node);
}

echo $callback, "|", $hook->callbacks[10]["id"]["function"], "|", $hook->callbacks[10]["id"]["accepted_args"], "|", $hook->callbacks[10]["plain"]["function"];
