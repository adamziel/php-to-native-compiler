<?php
error_reporting(0);

function milestone2075_replace_parent(&$value) {
    $value["ref"] = array("value" => "parent-copy");
    $value["plain"]["value"] = "plain-copy";
    return $value["ref"]["value"] . "|" . $value["plain"]["value"];
}

class Milestone2075_Box {
    public $alt = array();

    public function __get($name) {
        if ($name === "slot") {
            $bucket =& $this->alt[$name];
        }
        return $bucket;
    }
}

$box = new Milestone2075_Box();
$box->alt = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->alt["slot"]["ref"]["value"];

$result = milestone2075_replace_parent($box->slot);

echo $alias, "|", $box->alt["slot"]["ref"]["value"], "|",
    $box->alt["slot"]["plain"]["value"], "|", $result;
