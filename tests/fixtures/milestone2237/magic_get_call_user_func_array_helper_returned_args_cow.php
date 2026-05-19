<?php
error_reporting(0);

function &milestone2237_pick_ref(&$value) {
    return $value["ref"]["value"];
}

function milestone2237_args($copy) {
    return array($copy);
}

class Milestone2237_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $alias =& call_user_func_array(
            "milestone2237_pick_ref",
            milestone2237_args($bucket)
        );
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2237_Box();
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
