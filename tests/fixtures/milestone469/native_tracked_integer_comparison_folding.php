<?php
$sum = 1 + 2;
$seed = 2 + 2;
$flag = $seed === 4;
$bounded = $flag ? 3 : 4;

echo $sum == 3, "\n";
echo $sum != 4, "\n";
echo 2 < $sum, "\n";
echo 4 <= $sum, "\n";
echo $sum > 1, "\n";
echo $sum >= 4, "\n";
echo 1 < 2, "\n";
echo $bounded == 3;
