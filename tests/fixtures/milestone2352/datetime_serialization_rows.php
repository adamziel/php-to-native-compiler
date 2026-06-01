<?php
date_default_timezone_set("UTC");
$tz = new DateTimeZone("Europe/Oslo");
echo serialize($tz), "\n";
$tzCopy = unserialize(serialize($tz));
echo $tzCopy->getName(), "|", serialize($tzCopy->__serialize()), "\n";
$manualTz = new DateTimeZone("UTC");
$manualTz->__unserialize(array("timezone_type" => 1, "timezone" => "+0400"));
echo serialize($manualTz), "\n";
$dt = new DateTime("2005-07-14 22:30:41 GMT");
echo serialize($dt), "\n";
$round = unserialize(serialize($dt));
echo $round->format("U|T|Y-m-d H:i:s"), "\n";
$manualDate = new DateTime("@0");
$manualDate->__unserialize(array(
    "date" => "2005-07-14 22:30:41.000000",
    "timezone_type" => 2,
    "timezone" => "GMT",
));
echo $manualDate->format("U|T|Y-m-d H:i:s"), "\n";
echo (method_exists("DateTimeZone", "__serialize") ? "tz-method" : "tz-missing"), "|";
echo (method_exists("DateTime", "__unserialize") ? "dt-method" : "dt-missing"), "\n";
