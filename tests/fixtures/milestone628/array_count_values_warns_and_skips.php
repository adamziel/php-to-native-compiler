<?php
$items = ["ok", true, null, [], 2, "2", false, 2.0];
$counted = array_count_values($items);
print_r($counted);
echo count($counted), "\n";

$call = "array_count_values";
$again = $call(["x", [], "x"]);
echo count($again), "|", $again["x"];
