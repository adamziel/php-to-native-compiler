<?php
$items = [
    "Name" => "Ada",
    "name" => "lower",
    3 => "three",
    "MiXeD" => "mixed",
];

$positive = array_change_key_case($items, 2);
print_r($positive);

$negative = array_change_key_case($items, -1);
print_r($negative);

$call = "array_change_key_case";
$dynamic = $call($items, 42);
echo $dynamic["NAME"], "|", $dynamic["MIXED"], "|", $dynamic[3], "\n";

$lower = array_change_key_case($items, 0);
echo $lower["name"], "|", $lower["mixed"];
