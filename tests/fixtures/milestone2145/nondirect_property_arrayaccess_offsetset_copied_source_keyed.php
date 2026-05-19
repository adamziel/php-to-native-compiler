<?php
error_reporting(0);

class Milestone2145_Source {
    public $store = array();
}

class Milestone2145_Bag implements ArrayAccess {
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

class Milestone2145_Holder {
    public $bag;
}

$source = new Milestone2145_Source();
$source->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $source->store["slot"]["ref"];

$holders = array("h" => new Milestone2145_Holder());
$holders["h"]->bag = new Milestone2145_Bag();

$holders["h"]->bag["slot"] = $source->store["slot"];
$holders["h"]->bag["slot"]["ref"] = "new";
$holders["h"]->bag["slot"]["plain"] = "copy";

echo implode(",", $holders["h"]->bag->log), "|", $alias, "|", $source->store["slot"]["ref"], "|", $holders["h"]->bag->store["slot"]["plain"];
