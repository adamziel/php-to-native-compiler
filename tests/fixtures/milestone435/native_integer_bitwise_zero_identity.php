<?php
$value = 6 + 2;
$or_right = $value | 0;
$or_left = 0 | $value;
$xor_right = $value ^ 0;
$xor_left = 0 ^ $value;

echo $or_right + $or_left + $xor_right + $xor_left;
