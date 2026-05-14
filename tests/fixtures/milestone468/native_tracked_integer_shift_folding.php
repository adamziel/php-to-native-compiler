<?php
$base = 6 + 2;
$literal_left = 8 << 1;
$literal_right = 8 >> 1;
$seed = 1 + 2;
$flag = $seed === 3;
$bounded = $flag ? 5 : 6;

echo $base << 2, "\n";
echo $base >> 1, "\n";
echo $literal_left + $literal_right, "\n";
echo $bounded << 1;
