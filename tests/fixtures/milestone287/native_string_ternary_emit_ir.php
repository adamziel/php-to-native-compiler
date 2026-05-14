<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$first = $is_three ? "alpha" : "beta";
$second = $is_four ? "gamma" : $first;

echo $first, "\n";
echo $second, "!";
