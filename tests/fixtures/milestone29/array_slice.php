<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$slice = array_slice($items, 2);
echo count($slice), "\n";
echo $slice[0], "|", $slice["02"], "|", $slice[1], "|", $slice[2], "\n";
$slice[] = "after";
echo $slice[3], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6], "\n";

$tail = array_slice($items, -3);
echo count($tail), "|", $tail["02"], "|", $tail[0], "|", $tail[1], "\n";

$empty = array_slice($items, 99);
echo count($empty), "\n";

$whole = array_slice($items, -99);
echo count($whole), "|", $whole["name"], "|", $whole[0], "|", $whole[1], "|", $whole["02"], "|", $whole[2], "|", $whole[3], "\n";

$call = "array_slice";
$again = $call($items, 1);
echo count($again), "|", $again[0], "|", $again["02"], "|", $again[3];
