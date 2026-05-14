<?php
$sum = 1 + 2;
$left = $sum === 3;
$right = $sum !== 4;
$falsey = $sum === 4;
$choice = $left ? 3 : 4;
$amb_left = $sum === $choice;
$amb_right = $choice === 4;

echo $left && $right, "\n";
echo $left || $falsey, "\n";
echo $left xor $right, "\n";
echo $falsey && $right, "\n";
echo $amb_left && $amb_right;
