<?php
function &milestone1861_pick($name) {
    global $store;
    $GLOBALS["trace"][] = "pick:" . $name;
    return $store[$name];
}

$source = "seed";
$store = array("slot" => array("ref" => &$source, "plain" => array("value" => "copy")));
$GLOBALS["trace"] = array();

$alias =& milestone1861_pick("slot");
$alias["ref"] = "changed";

$copy = milestone1861_pick("slot");
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $store["slot"]["plain"]["value"], "|", implode(",", $GLOBALS["trace"]);
