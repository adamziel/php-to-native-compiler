<?php
$seed = 1 + 2;
$condition = $seed === 3;
$value = $condition ? 10 : 11;
$remainder = $value % 3;

echo $remainder + 5;
