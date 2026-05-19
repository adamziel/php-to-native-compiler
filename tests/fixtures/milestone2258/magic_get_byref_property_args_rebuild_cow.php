<?php
error_reporting(0);

function &milestone2258_pick_leaf(&$value) {
    return $value["ref"]["value"];
}

class Milestone2258_Box {
    public $store = array();
    public $args = array();

    public function prepare($copy, &$args) {
        $args = array($copy);
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->args = array();
        $this->prepare($bucket, $this->args);
        $alias =& call_user_func_array("milestone2258_pick_leaf", $this->args);
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2258_Box();
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
