<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$ambiguous = $sum === $choice;
$left = $ambiguous ? "alpha" : "bravo";
$middle = $ambiguous ? "charlie" : "delta";
$wide = $ambiguous ? $left : "echo";
$text = $ambiguous ? $wide : $middle;

echo ($text == $text) ? 1 : 0, "\n";
echo ($text != $text) ? 1 : 0, "\n";
echo ($text < $text) ? 1 : 0, "\n";
echo ($text <= $text) ? 1 : 0, "\n";
echo ($text > $text) ? 1 : 0, "\n";
echo ($text >= $text) ? 1 : 0;
