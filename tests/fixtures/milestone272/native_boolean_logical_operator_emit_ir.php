<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$choice = $is_three ? 3 : 4;
$left = $sum === $choice;
$right = $choice === 4;
$not_right = !$right;

echo $left && $right, "\n";
echo $left || $right, "\n";
echo $left xor $right, "\n";
echo $left && $not_right, "\n";
echo (true and false), "\n";
echo (false or true), "z";
