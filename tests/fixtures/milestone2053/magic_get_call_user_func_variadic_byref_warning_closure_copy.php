<?php
error_reporting(0);

class Milestone2053_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $helper = function (&...$values) {
            $values[0]["ref"]["value"] = "inside";
            $values[0]["plain"]["value"] = "inside-plain";
            return $values[0];
        };
        return call_user_func($helper, $bucket);
    }
}

$box = new Milestone2053_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
