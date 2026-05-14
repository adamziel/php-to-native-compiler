<?php
$items = [
    "s10" => "10",
    "i10" => 10,
    "f10" => 10.0,
    "s10f" => "10.0",
    "true" => true,
    "one" => 1,
    "false" => false,
    "empty" => "",
    "null" => null,
    "zero" => 0,
    "s0" => "0",
    "text" => "abc",
    "dup-text" => "abc",
];

$unique = array_unique($items, SORT_REGULAR);
print_r($unique);
echo $unique["s10"], "|", $unique["one"], "|", $unique["false"], "|", $unique["text"], "|", count($unique), "\n";

$call = "array_unique";
$again = $call($items, constant("SORT_REGULAR"));
echo $again["s10"], "|", $again["one"], "|", defined("SORT_REGULAR"), "|", SORT_REGULAR;
