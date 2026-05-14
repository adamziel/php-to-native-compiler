<?php
$value = 4 << 62;
$seed = 1 + 2;
$flag = $seed === 3;
$same = $flag ? $value : $value;

echo $same;
