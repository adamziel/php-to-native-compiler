<?php
$items = [
    "first" => "10",
    "second" => 10,
    "third" => "10.0",
    "fourth" => false,
    "fifth" => "",
    "sixth" => true,
    "seventh" => 1,
];

$unique = array_unique($items, SORT_STRING);
print_r($unique);
echo $unique["first"], "|", $unique["third"], "|", $unique["fourth"], "|", $unique["sixth"], "|", count($unique), "\n";

$call = "array_unique";
$again = $call($items, constant("SORT_STRING"));
echo $again["first"], "|", defined("SORT_STRING"), "|", SORT_STRING, "\n";
