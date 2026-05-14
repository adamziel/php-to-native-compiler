<?php
$keys = [null, false, true, "01"];
$values = ["null key", "false key", "true key", "string one"];

$combined = array_combine($keys, $values);
print_r($combined);
echo count($combined), "\n";
echo $combined[""], "|", $combined[1], "|", $combined["01"], "\n";

$call = "array_combine";
$again = $call($keys, $values);
echo $again[""], "|", $again[1], "|", $again["01"], "\n";
echo "done";
