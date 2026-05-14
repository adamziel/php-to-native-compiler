<?php
$seed = 1 + 2;
$flag = $seed === 3;
$sum = $flag ? 3.75 : 4.25;
echo $sum === 3.75, "\n";
echo $sum !== 4.25, "x";
