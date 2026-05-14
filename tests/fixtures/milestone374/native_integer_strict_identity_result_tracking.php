<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$choice = $is_three ? 3 : 4;
$ambiguous = $sum === $choice;

echo ($is_three === true) ? 1 : 0, "\n";
echo ($is_four === false) ? 1 : 0, "\n";
echo ($ambiguous === true) ? 1 : 0;
