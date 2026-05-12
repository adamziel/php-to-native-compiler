<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[] = "left next";

$right = [];
$right["name"] = "Bea";
$right[7] = "seven";
$right["02"] = "zero two right";
$right[] = "right next";
$right["extra"] = "extra";

$merged = array_merge($left, $right);
print_r($merged);
echo count($merged), "\n";
echo $merged["name"], "|", $merged[0], "|", $merged[1], "|", $merged["02"], "|", $merged[2], "|", $merged[3], "|", $merged[4], "|", $merged["extra"], "\n";
$merged[] = "after";
echo $merged[5], "\n";
print_r($left);
print_r($right);

$call = "array_merge";
$again = $call($left, $right);
echo $again["name"], "|", $again[0], "|", $again["02"], "|", $again["extra"];
