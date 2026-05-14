<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$both = $is_three && $is_three;
$either = $is_three || $is_three;

echo $both ? 1 : 0, "\n", $either ? 1 : 0;
