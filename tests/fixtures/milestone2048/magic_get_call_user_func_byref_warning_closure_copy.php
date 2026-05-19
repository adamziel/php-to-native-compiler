<?php
error_reporting(0);

class Milestone2048_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $helper = function (&$value) {
            $value["ref"]["value"] = "inside";
            $value["plain"]["value"] = "inside-plain";
            return $value;
        };
        return call_user_func($helper, $bucket);
    }
}

$box = new Milestone2048_Box();
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
