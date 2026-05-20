<?php
$source = array(
    "ref" => array("leaf" => "source"),
    "plain" => "source-plain",
);
$alias =& $source["ref"]["leaf"];
$copy = $source;

$copy["ref"]["leaf"] = "copy-ref";
$copy["plain"] = "copy-plain";

echo $source["ref"]["leaf"], "|", $alias, "|";
echo $source["plain"], "|", $copy["plain"];
