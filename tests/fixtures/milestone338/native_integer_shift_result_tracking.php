<?php
$left = 6 + 2;
$shift_left = $left << 2;
$shift_right = $shift_left >> 3;
$negative = -8;
$shift_negative = $negative >> 1;

echo $shift_left + 5, "\n";
echo $shift_right + $shift_negative;
