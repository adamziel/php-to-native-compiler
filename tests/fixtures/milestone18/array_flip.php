<?php
$items = [];
$items["first"] = "name";
$items[5] = "2";
$items["two"] = 2;
$items["02"] = "02";
$items[] = -1;
$items["dup-string"] = "name";

$flipped = array_flip($items);
print_r($flipped);
echo count($flipped), "\n";
echo $flipped["name"], "|", $flipped[2], "|", $flipped["02"], "|", $flipped[-1], "\n";
$flipped[] = "after";
echo $flipped[3], "\n";
print_r($items);

$call = "array_flip";
$again = $call($items);
echo $again["name"], "|", $again[2], "|", $again["02"], "|", $again[-1];
