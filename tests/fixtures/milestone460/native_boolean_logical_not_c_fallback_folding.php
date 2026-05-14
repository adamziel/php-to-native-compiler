<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;

$not_three = !$is_three;
$not_four = !$is_four;

echo $not_three ? 1 : 0, "\n";
echo $not_four ? 1 : 0;
