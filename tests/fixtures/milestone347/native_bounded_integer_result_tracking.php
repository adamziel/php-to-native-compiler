<?php
$sum = 1 + 2;
$left_flag = $sum === 3;
$left = $left_flag ? 1 : 2;

$right_sum = 2 + 2;
$right_flag = $right_sum === 4;
$right = $right_flag ? 10 : 20;

echo $left + $right;
