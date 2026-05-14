<?php
$sum = 1 + 2;
$flag = $sum === 3;
$always = $flag || true;
$never = false && $flag;
$same = $flag && true;
$also = ($flag xor false);
$invert = ($flag xor true);

echo $always ? 10 : 20, "\n";
echo $never ? 10 : 20, "\n";
echo $same ? 1 : 0, "\n";
echo $also ? 1 : 0, "\n";
echo $invert ? 1 : 0;
