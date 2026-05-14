<?php
$sum = 1 + 2;
$always_left = $sum === 3;
$always_right = $sum !== 4;
$choice = $always_left ? 3 : 4;
$ambiguous = $sum === $choice;

echo $always_left == $always_right, "\n";
echo $always_left != $always_right, "\n";
echo $always_left == $ambiguous;
