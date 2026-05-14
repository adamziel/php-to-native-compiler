<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$integer = $is_three ? 10 + 2 : 99;
$boolean = $is_four ? true : $is_three;

echo $integer, "\n";
echo $boolean ? 1 : 0, "z";
