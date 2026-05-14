<?php
$sum = 1 + 2;
$flag = $sum === 3;
$bounded = $flag ? 7 : 8;
$other = $flag ? 2 : 4;

echo $bounded > 6, "\n";
echo $bounded < 10, "\n";
echo 1 < $other, "\n";
echo $other >= $bounded, "\n";
echo $bounded == 7;
