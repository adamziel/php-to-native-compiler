<?php
$left = 1 + 2;
$right = 4 + 5;
$mul_left = 2 * 3;
$mul_right = 1 + 4;
$flag = $left === 3;
$amb_left = $flag ? 3 : 4;
$amb_right = $flag ? 5 : 6;

echo $left + $right, "\n";
echo $right - $left, "\n";
echo $mul_left * $mul_right, "\n";
echo $amb_left + $amb_right;
