<?php
error_reporting(0);

function &milestone2240_pick_ref(&$value) {
    return $value["ref"]["value"];
}

function milestone2240_args($copy) {
    return array($copy);
}

class Milestone2240_Box {
    public $store = array();

    public function helper($bucket) {
        $alias =& call_user_func_array(
            "milestone2240_pick_ref",
            milestone2240_args($bucket)
        );
        $alias = "inside";
        return $bucket;
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $method = "helper";
        return $this->{$method}($bucket);
    }
}

$box = new Milestone2240_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
