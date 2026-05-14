<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? 1.5 : 2.5;

echo $value !== 9.5, "\n";
echo ($value === 9.5) ? 10 : 20, "\n";
echo ($value === 1.5) ? 1 : 0;
