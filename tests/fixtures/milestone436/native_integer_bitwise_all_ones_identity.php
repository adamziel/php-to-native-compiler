<?php
$value = 6 + 2;
$all_ones = 0 - 1;
$and_right = $value & $all_ones;
$and_left = $all_ones & $value;

echo $and_right + $and_left;
