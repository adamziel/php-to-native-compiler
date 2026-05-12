<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$middle = array_slice($items, 1, 3);
echo count($middle), "|", $middle[0], "|", $middle[1], "|", $middle["02"], "\n";

$zero = array_slice($items, 1, 0);
echo count($zero), "\n";

$without_tail = array_slice($items, 1, -2);
echo count($without_tail), "|", $without_tail[0], "|", $without_tail[1], "|", $without_tail["02"], "\n";

$empty = array_slice($items, 4, -3);
echo count($empty), "\n";

$negative_offset = array_slice($items, -4, 2);
echo count($negative_offset), "|", $negative_offset[0], "|", $negative_offset["02"], "\n";

$call = "array_slice";
$dynamic = $call($items, 0, 2);
echo count($dynamic), "|", $dynamic["name"], "|", $dynamic[0], "\n";

echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
