<?php
$left = 6 + 2;
$mask = 3;
$and = $left & $mask;
$or = $left | $mask;
$xor = $or ^ $mask;
$not = ~$mask;

echo $and + 5, "\n";
echo $xor + $not;
