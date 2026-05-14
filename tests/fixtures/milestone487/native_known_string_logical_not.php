<?php
$sum = 1 + 2;
$flag = $sum === 3;
$truthy = $flag ? "left" : "right";
$falsey = $flag ? "" : "0";

echo !"" ? 1 : 0, "\n";
echo !"0" ? 1 : 0, "\n";
echo !"literal" ? 1 : 0, "\n";
echo !$truthy ? 1 : 0, "\n";
echo !$falsey ? 1 : 0;
