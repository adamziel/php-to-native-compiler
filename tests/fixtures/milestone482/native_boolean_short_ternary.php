<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$choice = $is_three ? 3 : 4;
$maybe_three = $sum === $choice;
$other_choice = $is_three ? 4 : 3;
$maybe_other = $sum === $other_choice;

echo ($is_three ?: false) ? 1 : 0, "\n";
echo ($maybe_three ?: $maybe_other) ? 1 : 0, "\n";
echo ($maybe_three ?: true) ? 1 : 0, "\n";
echo (false ?: $is_four) ? 1 : 0;
