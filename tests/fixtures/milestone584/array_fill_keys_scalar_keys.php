<?php
$keys = [null, false, true, 1.0, 2.0, "2", "02", -3.0];
$filled = array_fill_keys($keys, "x");
print_r($filled);
echo count($filled), "\n";
echo $filled[""], "|", $filled[1], "|", $filled[2], "|", $filled["02"], "|", $filled[-3], "\n";

$call = "array_fill_keys";
$again = $call([0.0, 1.0], "y");
echo count($again), "\n";
echo $again[0], "|", $again[1];
