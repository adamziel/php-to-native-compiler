<?php
$value = 1.25 + 2.5;
$seed = 2 + 2;
$flag = $seed === 4;
$bounded = $flag ? 3.75 : 4.75;

echo $value == 3.75, "\n";
echo $value != 4.25, "\n";
echo 2.5 < $value, "\n";
echo 3.5 <= $value, "\n";
echo $value > 1.25, "\n";
echo $value >= 4.0, "\n";
echo 1.25 < 2.5, "\n";
echo $bounded == 3.75;
