<?php
class Milestone1927_Box {
    public function __get($name) {
        return $GLOBALS["store"][$name];
    }
}

$GLOBALS["store"] = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $GLOBALS["store"]["slot"]["ref"]["value"];
$box = new Milestone1927_Box();

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $GLOBALS["store"]["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $GLOBALS["store"]["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
