<?php
error_reporting(0);

class Milestone2147_Source {
    public $store = array();
}

class Milestone2147_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        $key = $offset;
        if ($key === "slot") {
            $bucket = $this->store[$key];
        } else {
            $bucket = array();
        }
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->log[] = "set:" . $offset;
        $key = $offset;
        if ($key === "slot") {
            $payload = $value;
        } else {
            $payload = array();
        }
        $this->store[$key] = $payload;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone2147_Holder {
    public $bag;
    public $log = array();

    public function __get($name) {
        $this->log[] = "get:" . $name;
        return $this->bag;
    }
}

$source = new Milestone2147_Source();
$source->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $source->store["slot"]["ref"];

$holder = new Milestone2147_Holder();
$holder->bag = new Milestone2147_Bag();

$holder->missing["slot"] = $source->store["slot"];
$holder->missing["slot"]["ref"] = "new";
$holder->missing["slot"]["plain"] = "copy";

echo implode(",", $holder->log), "|", implode(",", $holder->bag->log), "|", $alias, "|", $source->store["slot"]["ref"], "|", $holder->bag->store["slot"]["plain"];
