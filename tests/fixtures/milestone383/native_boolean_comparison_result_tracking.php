<?php
$sum = 1 + 2;
$flag = $sum === 3;
$not_flag = !$flag;
$is_true = $flag == true;
$is_false = $not_flag == true;
$choice = $flag ? false : true;
$ambiguous = $flag == $choice;

echo $flag == true, "\n";
echo $not_flag < $flag, "\n";
echo ($is_true === true) ? 1 : 0, "\n";
echo ($is_false === false) ? 1 : 0, "\n";
echo ($ambiguous === false) ? 1 : 0;
