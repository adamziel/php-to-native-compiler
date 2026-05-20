<?php
error_reporting(0);

function &milestone2274_pick(&$value) {
    return $value["ref"]["value"];
}

class Milestone2274_Box {
    public $store = array();
    public $hits = 0;

    public function __set(string $name, mixed $value): void {
        $this->hits = $this->hits + 1;
        $tmp = $value;
        $this->store[$name] = $tmp;
    }
}

class Milestone2274_Source {
    public $store = array();
}

$source = new Milestone2274_Source();
$source->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $source->store["slot"]["ref"]["value"];

$bucket = $source->store["slot"];
$box = new Milestone2274_Box();
$box->args = array($bucket);

$alias =& call_user_func_array("milestone2274_pick", $box->store["args"]);
$alias = "inside";
$box->store["args"][0]["plain"]["value"] = "box-copy";

echo $ref, "|", $source->store["slot"]["plain"]["value"], "|", $box->store["args"][0]["ref"]["value"], "|", $box->hits;
