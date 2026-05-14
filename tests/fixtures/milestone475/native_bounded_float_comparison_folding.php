<?php
$sum = 1 + 2;
$flag = $sum === 3;
$bounded = $flag ? 7.5 : 8.5;
$other = $flag ? 2.5 : 4.5;

echo $bounded > 6.5, "\n";
echo $bounded < 10.5, "\n";
echo 1.5 < $other, "\n";
echo $other >= $bounded, "\n";
echo $bounded == 7.5;
