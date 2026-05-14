<?php
$sum = 1 + 2;
$flag = $sum === 3;
$same = $flag ? true : false;
$inverse = $flag ? false : true;
$always = $flag ? true : true;
$never = $flag ? false : false;

echo $same ? "T" : "F", "\n";
echo $inverse ? "T" : "F", "\n";
echo $always ? "T" : "F", "\n";
echo $never ? "T" : "F";
