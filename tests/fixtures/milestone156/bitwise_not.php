<?php
var_dump(~0);
var_dump(~5);
var_dump(~-1);
var_dump(~"");
var_dump(~1 & 3);

$value = 0;
var_dump(~($value = 4));
echo $value;
