<?php
$sum = 1 + 2;
$flag = $sum === 3;
$truthy = $flag ? "left" : "right";
$falsey = $flag ? "" : "0";

echo "literal" ?: [], "\n";
echo "" ?: "empty", "\n";
echo "0" ?: "zero", "\n";
echo $truthy ?: "fallback", "\n";
echo $falsey ?: "falsey";
