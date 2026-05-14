<?php
$value = 6 + 2;
$same_and = $value & $value;
$same_or = $value | $value;
$same_xor = $value ^ $value;

echo $same_and + $same_or, "\n";
echo $same_xor + 5;
