<?php
$source = array("outer" => array());
$copy = $source;
$value = "start";

$copy["outer"][] =& $value;
$value = "changed";

echo isset($source["outer"][0]) ? $source["outer"][0] : "no";
echo "|", $copy["outer"][0];
