<?php
$items = [];
$items["Name"] = "Ada";
$items["name"] = "lower";
$items[7] = "seven";
$items["MiXeD"] = "mixed";
$items["02"] = "numeric string";

$lower = array_change_key_case($items);
print_r($lower);
$upper = array_change_key_case($items, CASE_UPPER);
print_r($upper);
echo $lower["name"], "|", $lower[7], "|", $lower["mixed"], "|", $lower["02"], "\n";
echo $upper["NAME"], "|", $upper[7], "|", $upper["MIXED"], "|", $upper["02"], "\n";
$lower[] = "after";
echo $lower[8], "\n";
print_r($items);

$call = "array_change_key_case";
$again = $call($items, CASE_LOWER);
echo $again["name"], "|", constant("CASE_UPPER"), "|", defined("CASE_LOWER"), "\n";
