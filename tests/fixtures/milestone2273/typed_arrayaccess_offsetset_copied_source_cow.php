<?php
error_reporting(0);

function &milestone2273_pick(&$value) {
    return $value["ref"]["value"];
}

class Milestone2273_Holder implements ArrayAccess {
    public $store = array();
    public $hits = 0;

    public function offsetExists(mixed $offset): bool {
        return isset($this->store[$offset]);
    }

    public function offsetGet(mixed $offset): mixed {
        return $this->store[$offset];
    }

    public function offsetSet(mixed $offset, mixed $value): void {
        $this->hits = $this->hits + 1;
        $tmp = $value;
        $this->store[$offset] = $tmp;
    }

    public function offsetUnset(mixed $offset): void {
        unset($this->store[$offset]);
    }
}

class Milestone2273_Source {
    public $store = array();
}

$source = new Milestone2273_Source();
$source->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $source->store["slot"]["ref"]["value"];

$bucket = $source->store["slot"];
$holder = new Milestone2273_Holder();
$holder["args"] = array($bucket);

$alias =& call_user_func_array("milestone2273_pick", $holder["args"]);
$alias = "inside";
$holder->store["args"][0]["plain"]["value"] = "holder-copy";

echo $ref, "|", $source->store["slot"]["plain"]["value"], "|", $holder->store["args"][0]["ref"]["value"], "|", $holder->hits;
