<?php
$sum = 1 + 2;
$is_three = $sum === 3;

echo $is_three, "\n";
echo $is_three !== 1, "\n";
echo $is_three !== "1", "\n";
echo $is_three !== null, "\n";
echo $is_three !== 1.0, "\n";
echo "x", $is_three === 1, "y";
