<?php
error_reporting(0);

class Milestone2272_Box {
    public $store = array();
    public $hits = 0;

    public function __get(string $name): mixed {
        $this->hits = $this->hits + 1;
        $key = $name;
        return $this->store[$key];
    }
}

$box = new Milestone2272_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$box->slot["ref"]["value"] = "inside";
$box->slot["plain"]["value"] = "copy-only";

echo $ref, "|", $box->store["slot"]["plain"]["value"], "|", $box->hits;
