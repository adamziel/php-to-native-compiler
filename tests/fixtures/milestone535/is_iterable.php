<?php
class Box {}

$box = new Box();
$values = [null, false, true, 0, 3.5, "", [], [1], $box];
foreach ($values as $value) {
    echo is_iterable($value) ? "1" : "0";
}
echo "\n";

$call = "is_iterable";
echo $call([]) ? "1" : "0", $call("x") ? "1" : "0";
