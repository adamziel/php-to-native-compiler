<?php
$match = ["full-match", "default", "en_US"];
list(, $textdomain, $language) = $match;
echo $textdomain, "|", $language, "\n";

$values = ["left", "skip", "right"];
list($left, , $right,) = $values;
echo $left, "|", $right;
