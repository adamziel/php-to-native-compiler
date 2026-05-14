<?php
$value = 6 + 2;
$all_ones = 0 - 1;
$or_right = $value | $all_ones;
$or_left = $all_ones | $value;

echo $or_right + 0, "\n";
echo 0 + $or_left;
