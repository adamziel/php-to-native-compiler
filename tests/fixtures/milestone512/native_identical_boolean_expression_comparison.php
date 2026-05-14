<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$ambiguous = $sum === $choice;

echo ($ambiguous == $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous != $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous < $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous <= $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous > $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous >= $ambiguous) ? 1 : 0;
