<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[] = "next";

$values = array_values($items);
print_r($values);
echo count($values), "\n";
echo $values[0], "|", $values[1], "|", $values[2], "|", $values[3], "|", $values[4], "\n";

$call = "array_values";
$again = $call($items);
echo $again[2], "\n";
print_r($items);
