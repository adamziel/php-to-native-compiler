<?php
$seed = 1 + 2;
$flag = $seed === 3;
$value = $flag ? 5 : 6;
$mask = $flag ? 3 : 1;

$and = $value & $mask;
$shifted = $value << 1;
$negated = -$value;
$flipped = ~$mask;

echo $and + 10, "\n";
echo $shifted + $negated, "\n";
echo $flipped + 20;
