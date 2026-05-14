<?php
$sum = 1 + 2;
$flag = $sum === 3;
$other_sum = 2 + 2;
$other = $other_sum === 4;
$float_sum = 1.25 + 2.5;
$int = $other ? $sum : 3;
$float = $other ? $float_sum : 3.75;
$bool = $other ? $flag : true;
$ambiguous = $other ? 10 : 20;

echo $int + 4, "\n";
echo $float + 1.25, "\n";
echo $bool ? 1 : 0, "\n";
echo $ambiguous + 1;
