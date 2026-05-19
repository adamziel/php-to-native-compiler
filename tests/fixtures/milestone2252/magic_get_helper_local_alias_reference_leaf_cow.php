<?php
error_reporting(0);

class Milestone2252_Box {
    public $store = array();

    public function mutate($bucket) {
        $alias =& $bucket["ref"]["value"];
        $alias = "copy-ref";
        $bucket["plain"]["value"] = "copy-plain";
        return $bucket;
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        return $this->mutate($bucket);
    }
}

$box = new Milestone2252_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $box->store["slot"]["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
