<?php
$source = array(
    "plain" => array("leaf" => "source"),
    "sibling" => "keep",
);
$copy = $source;

$copy["plain"]["leaf"] = "copy";
$copy["plain"]["new"] = "copy-new";

echo $source["plain"]["leaf"], "|";
echo isset($source["plain"]["new"]) ? "yes" : "no", "|";
echo $copy["plain"]["leaf"], "|", $copy["plain"]["new"];
