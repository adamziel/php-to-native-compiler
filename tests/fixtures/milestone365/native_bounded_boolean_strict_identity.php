<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? true : true;
$inverse = !$value;
$ambiguous = $flag ? true : false;

echo $value === true, "\n";
echo ($inverse === true) ? 10 : 20, "\n";
echo ($ambiguous === true) ? 1 : 0;
