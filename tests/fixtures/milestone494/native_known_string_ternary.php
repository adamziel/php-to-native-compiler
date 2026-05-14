<?php
$sum = 1 + 2;
$flag = $sum === 3;
$truthy = $flag ? "left" : "right";
$falsey = $flag ? "" : "0";

echo "literal" ? "one" : "bad", "\n";
echo "" ? "bad" : "empty", "\n";
echo "0" ? "bad" : "zero", "\n";
echo $truthy ? 7 : 9, "\n";
echo $falsey ? "bad" : "falsey";
