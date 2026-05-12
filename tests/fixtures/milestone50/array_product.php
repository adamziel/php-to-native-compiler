<?php
$integers = [true, 2, " 4 ", "-3"];
echo array_product($integers), "\n";

$with_zero = [null, true, 2];
echo array_product($with_zero), "\n";

$mixed = [];
$mixed["int"] = 2;
$mixed["float"] = 3.5;
$mixed["exponent"] = "6e1";
$mixed["decimal"] = ".25";
echo array_product($mixed), "\n";

$empty = [];
echo array_product($empty), "\n";
echo $mixed["exponent"], "|", $mixed["decimal"], "\n";

$call = "array_product";
echo $call($mixed);
