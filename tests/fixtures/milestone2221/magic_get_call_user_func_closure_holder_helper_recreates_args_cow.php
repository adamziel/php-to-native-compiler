<?php
error_reporting(0);

function &milestone2221_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2221_ArgsHolder {
    public $args = array();
}

class Milestone2221_Box {
    public $store = array();
    public $holder;
    public $calls = 0;

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->holder->args = array();
        $helper = function ($copy) {
            $this->calls = $this->calls + 1;
            unset($this->holder->args);
            $this->holder->args = array($copy);
            return $this->holder;
        };
        $alias =& call_user_func_array(
            "milestone2221_pick_ref",
            call_user_func($helper, $bucket)->args
        );
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2221_Box();
$box->holder = new Milestone2221_ArgsHolder();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $box->calls, ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $box->calls;
