<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$preserved = array_slice($items, 1, 3, true);
echo count($preserved), "|", $preserved[5], "|", $preserved[2], "|", $preserved["02"], "\n";
$preserved[] = "after";
echo $preserved[6], "\n";

$default_false = array_slice($items, 1, 3, false);
echo count($default_false), "|", $default_false[0], "|", $default_false[1], "|", $default_false["02"], "\n";

$tail = array_slice($items, -3, null, true);
echo count($tail), "|", $tail["02"], "|", $tail[-1], "|", $tail[6], "\n";

$call = "array_slice";
$dynamic = $call($items, 0, null, true);
echo count($dynamic), "|", $dynamic["name"], "|", $dynamic[5], "|", $dynamic[2], "|", $dynamic["02"], "|", $dynamic[-1], "|", $dynamic[6], "\n";

echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
