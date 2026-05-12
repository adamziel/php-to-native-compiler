<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[-1] = "negative";
$left["drop"] = "drop";
$left[] = "next";

$right = [];
$right["name"] = "ignored";
$right["5"] = "ignored";
$right[2] = "ignored";
$right["02"] = "ignored";
$right[-1] = "ignored";
$right["extra"] = "ignored";

$intersected = array_intersect_key($left, $right);
print_r($intersected);
echo count($intersected), "\n";
echo $intersected["name"], "|", $intersected[5], "|", $intersected[2], "|", $intersected["02"], "|", $intersected[-1], "\n";
$intersected[] = "after";
echo $intersected[6], "\n";
print_r($left);
print_r($right);

$call = "array_intersect_key";
$again = $call($left, $right);
echo $again["name"], "|", $again[5], "|", $again[2], "|", $again["02"], "|", $again[-1], "\n";

$empty = array_intersect_key([], $right);
print_r($empty);
echo count($empty), "\n";

$none = array_intersect_key(["missing" => "x"], $right);
print_r($none);
echo count($none), "\n";
echo "done";
