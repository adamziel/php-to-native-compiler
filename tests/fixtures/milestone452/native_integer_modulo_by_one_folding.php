<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? 10 : 11;
$tracked = $value % 1;
$literal = 17 % 1;

echo $tracked + 5, "\n";
echo $literal + 7;
