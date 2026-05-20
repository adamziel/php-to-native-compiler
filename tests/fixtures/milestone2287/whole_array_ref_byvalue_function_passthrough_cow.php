<?php
function lane2287_passthrough($value) {
    return $value;
}

$source = array(
    "bucket" => array("leaf" => "source"),
    "plain" => array("leaf" => "plain-source"),
);
$bucket =& $source["bucket"];
$copy = lane2287_passthrough($source);

$copy["bucket"]["leaf"] = "copy-ref";
$copy["plain"]["leaf"] = "copy-plain";

echo $source["bucket"]["leaf"], "|", $bucket["leaf"], "|", $copy["bucket"]["leaf"], "|";
echo $source["plain"]["leaf"], "|", $copy["plain"]["leaf"];
