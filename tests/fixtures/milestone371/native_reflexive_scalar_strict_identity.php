<?php
$sum = 1 + 2;
$flag = $sum === 3;
$int = $flag ? 5 : 6;
$float = $flag ? 1.5 : 2.5;
$word = $flag ? "left" : "right";
$bool = $flag ? true : false;

echo ($int === $int) ? 1 : 0, "\n";
echo ($float !== $float) ? 1 : 0, "\n";
echo ($word === $word) ? 1 : 0, "\n";
echo ($bool !== $bool) ? 1 : 0;
