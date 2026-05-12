<?php
$integers = [null, false, true, 2, " 4 ", "-3"];
echo array_sum($integers), "\n";

$mixed = [];
$mixed["int"] = 2;
$mixed["float"] = 3.5;
$mixed["exponent"] = "6e1";
$mixed["decimal"] = ".25";
echo array_sum($mixed), "\n";

$empty = [];
echo array_sum($empty), "\n";
echo $mixed["exponent"], "|", $mixed["decimal"], "\n";

$call = "array_sum";
echo $call($mixed);
