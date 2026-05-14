<?php
$seed = 1 + 2;
$flag = $seed === 3;
$value = $flag ? 5 : 6;

echo $value !== 7, "\n";
echo ($value === 7) ? 10 : 20;
