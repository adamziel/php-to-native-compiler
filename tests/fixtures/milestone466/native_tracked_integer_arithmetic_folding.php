<?php
$base = 1 + 2;
$other = 4 + 5;
$flag = $base === 3;
$left = $flag ? 3 : 4;
$right = $flag ? 5 : 6;

echo $base + 4, "\n";
echo 10 - $base, "\n";
echo $base * 5, "\n";
echo 1 + 2, "\n";
echo $base + $other, "\n";
echo $left + $right;
