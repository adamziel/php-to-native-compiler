<?php
error_reporting(0);

function milestone2074_mutate(&$value) {
    $value["ref"]["value"] = "copy";
    $value["plain"]["value"] = "plain-copy";
    return $value["ref"]["value"] . "|" . $value["plain"]["value"];
}

class Milestone2074_Box {
    public $store = array();
    public $alt = array();

    public function __get($name) {
        if ($name === "group") {
            $bucket =& $this->alt[$name];
        } else {
            $bucket =& $this->store[$name];
        }
        return $bucket;
    }
}

$box = new Milestone2074_Box();
$box->store = array(
    "group" => array(
        "slot" => array(
            "ref" => array("value" => "store"),
            "plain" => array("value" => "store-plain"),
        ),
    ),
);
$box->alt = array(
    "group" => array(
        "slot" => array(
            "ref" => array("value" => "original"),
            "plain" => array("value" => "plain-original"),
        ),
    ),
);
$alias =& $box->alt["group"]["slot"]["ref"]["value"];
$storeAlias =& $box->store["group"]["slot"]["ref"]["value"];

$result = milestone2074_mutate($box->group["slot"]);

echo $alias, "|", $box->alt["group"]["slot"]["ref"]["value"], "|", $storeAlias, "|",
    $box->alt["group"]["slot"]["plain"]["value"], "|", $result;
