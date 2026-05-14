<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$first = $is_three ? 1.5 : 2.5;
$second = $is_four ? 9.25 : $first;

echo $first, "\n";
echo $second, "z";
