<?php
error_reporting(0);

function &milestone2238_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2238_ArgsHolder {
    public $args = array();
}

function milestone2238_holder($copy) {
    $holder = new Milestone2238_ArgsHolder();
    $holder->args = array(&$copy);
    return $holder;
}

class Milestone2238_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $holder = milestone2238_holder($bucket);
        $alias =& call_user_func_array("milestone2238_pick_ref", $holder->args);
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2238_Box();
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
