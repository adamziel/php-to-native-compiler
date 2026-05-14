<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$same = $is_four ? $is_three : $is_three;

echo $same ? 1 : 0;
