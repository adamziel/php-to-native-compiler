<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$maybe = $sum === $choice;

echo $maybe === false, "\n";
echo false === $maybe, "\n";
echo $maybe !== true, "\n";
echo true !== $maybe;
