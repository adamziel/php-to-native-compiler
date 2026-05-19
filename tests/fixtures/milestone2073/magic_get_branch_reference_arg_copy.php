<?php
error_reporting(0);

function milestone2073_mutate(&$value) {
    $value["ref"]["value"] = "copy";
    $value["plain"]["value"] = "plain-copy";
    return $value["ref"]["value"] . "|" . $value["plain"]["value"];
}

class Milestone2073_Box {
    public $store = array();
    public $alt = array();

    public function __get($name) {
        if ($name === "slot") {
            $bucket =& $this->alt[$name];
        } else {
            $bucket =& $this->store[$name];
        }
        return $bucket;
    }
}

$box = new Milestone2073_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "store"),
        "plain" => array("value" => "store-plain"),
    ),
);
$box->alt = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->alt["slot"]["ref"]["value"];
$storeAlias =& $box->store["slot"]["ref"]["value"];

$result = milestone2073_mutate($box->slot);

echo $alias, "|", $box->alt["slot"]["ref"]["value"], "|", $storeAlias, "|",
    $box->alt["slot"]["plain"]["value"], "|", $result;
