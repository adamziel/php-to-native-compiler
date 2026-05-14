<?php
$sum = 1 + 2;
$text = "3";
$nil = null;
$flag = true;
$float = 3.0;

echo $sum !== $float, "\n";
echo $text !== $sum, "\n";
echo $nil !== false, "\n";
echo $flag !== 1, "\n";
echo "x", $sum === $float, "y";
