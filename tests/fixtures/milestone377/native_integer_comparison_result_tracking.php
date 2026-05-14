<?php
$sum = 1 + 2;
$limit = $sum + 3;
$is_three = $sum === 3;
$is_small = $sum < 4;
$is_large = $sum > 9;
$choice = $is_three ? 4 : 2;
$ambiguous = $sum < $choice;

echo $sum == 3, "\n";
echo $limit >= 6, "\n";
echo ($is_small === true) ? 1 : 0, "\n";
echo ($is_large === false) ? 1 : 0, "\n";
echo ($ambiguous === true) ? 1 : 0;
