<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$tail = array_slice($items, 1, null);
echo count($tail), "|", $tail[0], "|", $tail[1], "|", $tail["02"], "|", $tail[2], "|", $tail[3], "\n";

$negative = array_slice($items, -3, null);
echo count($negative), "|", $negative["02"], "|", $negative[0], "|", $negative[1], "\n";

$empty = array_slice($items, 99, null);
echo count($empty), "\n";

$call = "array_slice";
$dynamic = $call($items, 0, null);
echo count($dynamic), "|", $dynamic["name"], "|", $dynamic[0], "|", $dynamic[1], "|", $dynamic["02"], "|", $dynamic[2], "|", $dynamic[3], "\n";

echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
