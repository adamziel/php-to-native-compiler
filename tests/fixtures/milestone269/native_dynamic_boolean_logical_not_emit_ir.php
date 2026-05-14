<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$choice = $is_three ? 3 : 4;
$other_choice = $is_three ? 4 : 3;
$maybe = $sum === $choice;
$maybe_other = $sum === $other_choice;
$not_maybe = !$maybe;
$not_other = !$maybe_other;

echo "a", $not_maybe, "b\n";
echo $not_other, "\n";
echo $not_maybe === false, "\n";
echo "x", !$not_other, "y";
