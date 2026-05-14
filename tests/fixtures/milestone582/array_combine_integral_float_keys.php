<?php
$keys = [1.0, 2.0, -3.0, "04"];
$values = ["one", "two", "minus", "leading"];

$combined = array_combine($keys, $values);
print_r($combined);
echo count($combined), "\n";
echo $combined[1], "|", $combined[2], "|", $combined[-3], "|", $combined["04"], "\n";

$call = "array_combine";
$again = $call([0.0, 1.0], ["zero", "one again"]);
echo $again[0], "|", $again[1];
