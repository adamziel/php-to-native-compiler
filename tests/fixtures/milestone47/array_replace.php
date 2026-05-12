<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[] = "left next";

$right = [];
$right["name"] = "Bea";
$right["5"] = "five right";
$right[7] = "seven";
$right["02"] = "zero two right";
$right[] = "right next";
$right["extra"] = "extra";

$replaced = array_replace($left, $right);
print_r($replaced);
echo count($replaced), "\n";
echo $replaced["name"], "|", $replaced[5], "|", $replaced[2], "|", $replaced["02"], "|", $replaced[6], "|", $replaced[7], "|", $replaced[8], "|", $replaced["extra"], "\n";
$replaced[] = "after";
echo $replaced[9], "\n";
print_r($left);
print_r($right);

$call = "array_replace";
$again = $call($left, $right);
echo $again["name"], "|", $again[5], "|", $again["02"], "|", $again["extra"], "\n";

$empty_replacement = array_replace($left, []);
print_r($empty_replacement);
echo count($empty_replacement), "\n";
$empty_replacement[] = "after empty";
echo $empty_replacement[7];
