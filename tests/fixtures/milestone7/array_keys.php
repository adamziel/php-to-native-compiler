<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

$keys = array_keys($items);
print_r($keys);
echo count($keys), "\n";
echo $keys[0], "|", $keys[1], "|", $keys[2], "|", $keys[3], "|", $keys[4], "|", $keys[5], "\n";

$call = "array_keys";
$again = $call($items);
echo $again[0], "|", $again[5], "\n";
print_r($items);
