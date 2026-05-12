<?php
$items = [];
$items["first"] = "name";
$items[5] = "2";
$items["two"] = 2;
$items["02"] = "02";
$items[] = -1;
$items["dup-string"] = "name";
$items["dup-int"] = 2;

$counted = array_count_values($items);
print_r($counted);
echo count($counted), "\n";
echo $counted["name"], "|", $counted[2], "|", $counted["02"], "|", $counted[-1], "\n";
$counted[] = "after";
echo $counted[3], "\n";
print_r($items);

$call = "array_count_values";
$again = $call($items);
echo $again["name"], "|", $again[2], "|", $again["02"], "|", $again[-1];
