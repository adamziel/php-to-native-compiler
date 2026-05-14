<?php
$left = 1.5 + 2.25;
$right = 4.0 + 0.5;
$mul_left = 1.25 + 0.25;
$mul_right = 2.0 + 1.0;
$zero_left = 1.5 + 2.25;
$zero_right = 3.0 + 0.75;
$seed = 1 + 2;
$flag = $seed === 3;
$amb_left = $flag ? 1.25 : 2.25;
$amb_right = $flag ? 2.75 : 3.75;

echo $left + $right, "\n";
echo $right - $left, "\n";
echo $mul_left * $mul_right, "\n";
echo 1.5 + 2.25, "\n";
echo $zero_left - $zero_right, "\n";
echo $amb_left + $amb_right;
