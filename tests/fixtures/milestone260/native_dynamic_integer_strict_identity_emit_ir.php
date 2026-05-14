<?php
$sum = 1 + 2;
$product = 3 * 2;
$same = $sum;

echo $sum === 3, "\n";
echo $product !== 6, "\n";
echo $sum !== $product, "\n";
echo "x", $same === $sum, "y";
