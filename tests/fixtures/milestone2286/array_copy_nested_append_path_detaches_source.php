<?php
$source = array("outer" => array("source"));
$copy = $source;

$copy["outer"][] = "copy";

echo isset($source["outer"][1]) ? $source["outer"][1] : "no";
echo "|", $copy["outer"][1], "|", $source["outer"][0];
