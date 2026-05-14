<?php
$sum = 1 + 2;
$flag = $sum === 3;
$word = $flag ? "alpha" : "beta";
$is_word = $word != "gamma";
$is_missing = $word == "gamma";
$choice = $flag ? "alpha" : "gamma";
$ambiguous = $word == $choice;

echo $word == "alpha", "\n";
echo $word < "gamma", "\n";
echo ($is_word === true) ? 1 : 0, "\n";
echo ($is_missing === false) ? 1 : 0, "\n";
echo ($ambiguous === true) ? 1 : 0;
