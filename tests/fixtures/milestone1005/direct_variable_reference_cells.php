<?php
$value = 1;
$alias =& $value;
$alias = 2;
echo $value;
echo "|";
$value = 3;
echo $alias;
unset($alias);
$value = 4;
echo "|";
echo $value;
$left = 5;
$right =& $left;
unset($left);
echo "|";
echo $right;
