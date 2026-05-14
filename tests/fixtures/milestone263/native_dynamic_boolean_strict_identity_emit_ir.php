<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$choice = $is_three ? 3 : 4;
$other_choice = $is_three ? 4 : 3;
$maybe = $sum === $choice;
$maybe_other = $sum === $other_choice;
$same = $is_three;

echo $maybe === true, "\n";
echo $maybe_other !== true, "\n";
echo $maybe !== $maybe_other, "\n";
echo "x", $same === $is_three, "y";
