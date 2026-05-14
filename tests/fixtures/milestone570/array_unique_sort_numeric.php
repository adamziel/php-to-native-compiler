<?php
$items = [
    "first" => "10",
    "second" => 10,
    "third" => "10.0",
    "fourth" => 10.5,
    "fifth" => "010.50",
    "sixth" => 11,
    "seventh" => "11.0",
    "eighth" => 0,
    "ninth" => false,
    "tenth" => null,
];

$unique = array_unique($items, SORT_NUMERIC);
print_r($unique);
echo $unique["first"], "|", $unique["fourth"], "|", $unique["sixth"], "|", $unique["eighth"], "|", count($unique), "\n";

$call = "array_unique";
$again = $call($items, constant("SORT_NUMERIC"));
echo $again["first"], "|", defined("SORT_NUMERIC"), "|", SORT_NUMERIC;
