<?php
class Milestone1932_Box {
    public function __get($name) {
        return $_REQUEST["store"][$name];
    }
}

$_REQUEST["store"] = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $_REQUEST["store"]["slot"]["ref"]["value"];
$box = new Milestone1932_Box();

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $_REQUEST["store"]["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $_REQUEST["store"]["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
