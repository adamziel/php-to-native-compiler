<?php
error_reporting(0);

function &milestone2255_pick_leaf(&$value) {
    return $value["ref"]["value"];
}

function milestone2255_make_holder($copy) {
    $holder = new stdClass();
    $holder->args = array($copy);
    return $holder;
}

class Milestone2255_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $holder = milestone2255_make_holder($bucket);
        $alias =& call_user_func_array("milestone2255_pick_leaf", $holder->args);
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2255_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
