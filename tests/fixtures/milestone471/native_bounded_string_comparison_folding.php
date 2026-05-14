<?php
$sum = 1 + 2;
$flag = $sum === 3;
$word = $flag ? "alpha" : "beta";
$choice = $flag ? "alpha" : "gamma";

echo $word != "gamma", "\n";
echo $word < "gamma", "\n";
echo "aardvark" < $word, "\n";
echo $word >= "alpha", "\n";
echo "zeta" > $word, "\n";
echo $word == $choice;
