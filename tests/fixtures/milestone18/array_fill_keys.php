<?php
$keys = [];
$keys["first"] = "name";
$keys[5] = "2";
$keys["two"] = 2;
$keys["02"] = "02";
$keys[] = -1;
$keys["dup-string"] = "name";

$filled = array_fill_keys($keys, "value");
print_r($filled);
echo count($filled), "\n";
echo $filled["name"], "|", $filled[2], "|", $filled["02"], "|", $filled[-1], "\n";
$filled[] = "after";
echo $filled[3], "\n";
print_r($keys);

$call = "array_fill_keys";
$again = $call($keys, "again");
echo $again["name"], "|", $again[2], "|", $again["02"], "|", $again[-1];
