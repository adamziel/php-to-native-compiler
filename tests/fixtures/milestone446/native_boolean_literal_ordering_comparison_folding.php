<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$maybe = $sum == $choice;

echo ($maybe < true) ? "T" : "F", "\n";
echo ($maybe > false) ? "T" : "F", "\n";
echo ($maybe <= true) ? "T" : "F", "\n";
echo ($maybe >= false) ? "T" : "F", "\n";
echo (false < $maybe) ? "T" : "F", "\n";
echo (true > $maybe) ? "T" : "F", "\n";
echo (false <= $maybe) ? "T" : "F", "\n";
echo (true >= $maybe) ? "T" : "F";
