<?php
function lane2287_plain_guard($value) {
    return $value;
}

$source = array(
    "bucket" => array("leaf" => "source"),
    "plain" => array("leaf" => "plain-source"),
);
$copy = lane2287_plain_guard($source);

$copy["bucket"]["leaf"] = "copy";
$copy["plain"]["leaf"] = "plain-copy";

echo $source["bucket"]["leaf"], "|", $copy["bucket"]["leaf"], "|";
echo $source["plain"]["leaf"], "|", $copy["plain"]["leaf"];
