<?php
var_dump(8 << 1);
var_dump(8 >> 1);
var_dump(-8 >> 1);
var_dump(1 + 2 << 3);
var_dump(1 << 2 + 1);
var_dump("x" . 1 << 2);
var_dump(1 << 2 < 8);
var_dump("8" << true);
var_dump(null >> 1);
var_dump(8 << 64);
var_dump(-1 >> 64);

$value = 0;
var_dump(($value = 4) << 2);
echo $value;
