<?php
$keys = [];
$keys["first"] = "name";
$keys[5] = "2";
$keys["two"] = 2;
$keys["02"] = "02";
$keys[] = -1;
$keys["dup-string"] = "name";

$values = [];
$values["a"] = "Ada";
$values[10] = "two string";
$values[] = "two int";
$values["d"] = "zero two";
$values[-3] = "negative";
$values[] = "duplicate";

$combined = array_combine($keys, $values);
print_r($combined);
echo count($combined), "\n";
echo $combined["name"], "|", $combined[2], "|", $combined["02"], "|", $combined[-1], "\n";
$combined[] = "after";
echo $combined[3], "\n";
print_r($keys);
print_r($values);

$call = "array_combine";
$again = $call($keys, $values);
echo $again["name"], "|", $again[2], "|", $again["02"], "|", $again[-1], "\n";

$empty = array_combine([], []);
print_r($empty);
echo count($empty), "\n";
echo "done";
