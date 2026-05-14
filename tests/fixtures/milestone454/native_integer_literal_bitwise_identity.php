<?php
$same_and = 5 & 5;
$same_or = 6 | 6;
$same_xor = 7 ^ 7;
$and_zero_right = 8 & 0;
$and_zero_left = 0 & 9;
$and_all_ones_right = 10 & -1;
$and_all_ones_left = -1 & 11;
$or_zero_right = 12 | 0;
$or_zero_left = 0 | 13;
$xor_zero_right = 14 ^ 0;
$xor_zero_left = 0 ^ 15;

echo $same_and, "\n";
echo $same_or, "\n";
echo $same_xor, "\n";
echo $and_zero_right, "\n";
echo $and_zero_left, "\n";
echo $and_all_ones_right, "\n";
echo $and_all_ones_left, "\n";
echo $or_zero_right, "\n";
echo $or_zero_left, "\n";
echo $xor_zero_right, "\n";
echo $xor_zero_left;
