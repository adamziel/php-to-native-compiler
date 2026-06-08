<?php
$items = [
    "Name" => "Ada",
    "name" => "lower",
    3 => "three",
    "MiXeD" => "mixed",
];

foreach ([-10, -1, 2] as $case) {
    try {
        array_change_key_case($items, $case);
    } catch (ValueError $e) {
        echo $e::class, ": ", $e->getMessage(), "\n";
    }
}

$call = "array_change_key_case";
$upper = $call($items, CASE_UPPER);
print_r($upper);

$lower = array_change_key_case($items, 0);
echo $lower["name"], "|", $lower["mixed"], "|", $lower[3];
