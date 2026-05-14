<?php
$left = 1.25 + 2.5;
$right = $left + 1.0;
$flag = $left == 3.75;
$is_less = $left < $right;
$is_too_high = $right > 9.0;
$choice = $flag ? 3.0 : 5.0;
$ambiguous = $left < $choice;

echo $left == 3.75, "\n";
echo $right >= 4.75, "\n";
echo ($is_less === true) ? 1 : 0, "\n";
echo ($is_too_high === false) ? 1 : 0, "\n";
echo ($ambiguous === false) ? 1 : 0;
