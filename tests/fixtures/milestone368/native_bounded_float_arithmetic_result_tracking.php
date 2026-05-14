<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? 1.25 : 1.25;
$offset = $flag ? 2.75 : 2.75;
$total = $value + $offset;
$ambiguous = ($flag ? 1.25 : 2.25) + 2.75;

echo ($total === 4.0) ? 10 : 20, "\n";
echo ($ambiguous === 4.0) ? 1 : 0;
