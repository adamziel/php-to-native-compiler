<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

$reversed = array_reverse($items);
print_r($reversed);
echo count($reversed), "\n";
echo $reversed[0], "|", $reversed[1], "|", $reversed["02"], "|", $reversed[2], "|", $reversed[3], "|", $reversed["name"], "\n";
$reversed[] = "after";
echo $reversed[4], "\n";
print_r($items);

$call = "array_reverse";
$again = $call($items);
echo $again[0], "|", $again["name"];
