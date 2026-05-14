<?php
$base = 6 + 2;
$other = 4 + 1;
$flag = $base === 8;
$left = $flag ? 12 : 10;
$right = $flag ? 5 : 3;

echo $base & 3, "\n";
echo 1 | $base, "\n";
echo $base ^ 5, "\n";
echo 6 & 3, "\n";
echo $base & $other, "\n";
echo $left & $right;
