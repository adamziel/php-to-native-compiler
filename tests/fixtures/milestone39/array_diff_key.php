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
$right[-1] = "ignored";
$right["extra"] = "ignored";

$diffed = array_diff_key($left, $right);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed["02"], "|", $diffed["drop"], "|", $diffed[6], "\n";
$diffed[] = "after";
echo $diffed[7], "\n";
print_r($left);
print_r($right);

$call = "array_diff_key";
$again = $call($left, $right);
echo $again["02"], "|", $again["drop"], "|", $again[6], "\n";

$empty = array_diff_key([], $right);
print_r($empty);
echo count($empty), "\n";

$all = array_diff_key(["missing" => "x"], []);
print_r($all);
echo count($all), "\n";

$none = array_diff_key(["name" => "x"], $right);
print_r($none);
echo count($none), "\n";
echo "done";
