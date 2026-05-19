<?php
error_reporting(0);

class Milestone2130_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        return $this->make($offset);
    }

    public function make($offset) {
        $this->log[] = "make:" . $offset;
        return array(
            "ref" => &$this->store[$offset]["ref"],
            "plain" => $this->store[$offset]["plain"],
        );
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$ref = "old";
$bag = new Milestone2130_Bag();
$bag->store["slot"] = array("ref" => &$ref, "plain" => "plain-original");
$bag["slot"]["ref"] = "new";

echo implode(",", $bag->log), "|", $ref, "|", $bag->store["slot"]["plain"];
