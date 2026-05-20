<?php
$source = array("outer" => array("keep" => "source"));
$copy = $source;

$alias =& $copy["outer"]["new"];
$alias = "copy";

echo isset($source["outer"]["new"]) ? $source["outer"]["new"] : "no";
echo "|", $copy["outer"]["new"], "|", $source["outer"]["keep"];
