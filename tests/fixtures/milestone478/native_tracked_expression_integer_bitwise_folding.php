<?php
$left = 6 + 2;
$right = 4 + 1;
$xor_left = 9 + 3;
$xor_right = 1 + 2;
$flag = $left === 8;
$amb_left = $flag ? 12 : 10;
$amb_right = $flag ? 5 : 3;

echo $left & $right, "\n";
echo $left | $right, "\n";
echo $xor_left ^ $xor_right, "\n";
echo 6 & 3, "\n";
echo $amb_left & $amb_right;
