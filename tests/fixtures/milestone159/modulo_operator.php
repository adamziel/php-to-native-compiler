<?php
var_dump(7 % 3);
var_dump(-7 % 3);
var_dump(7 % -3);
var_dump("8" % true);
var_dump(null % 3);
var_dump(10 % 4 * 2);
var_dump(10 % 4 + 1);
var_dump("x" . 5 % 2);
var_dump(5 % 2 == 1);

$value = 0;
var_dump(($value = 11) % 4);
echo $value;
