<?php
$sum = 1 + 2;
$flag = $sum === 3;
$left = $flag ? "alpha-2" : "beta_2";
$right = $flag ? "alpha-10" : "beta_10";
$is_later = $left > $right;
$is_missing = $left == "gamma!";
$choice = $flag ? "alpha-2" : "zeta!";
$ambiguous = $left == $choice;

echo $left > $right, "\n";
echo "a 1" < "a 2", "\n";
echo ($is_later === true) ? 1 : 0, "\n";
echo ($is_missing === false) ? 1 : 0, "\n";
echo ($ambiguous === true) ? 1 : 0;
