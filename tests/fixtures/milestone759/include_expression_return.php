<?php
$path = 'return_value.inc';
$value = include $path;
echo "include=", $value, ",", $side, "\n";

$normal = include 'normal.inc';
echo "normal=", $normal, ",", $normal_side, "\n";

$count = 0;
$once_first = include_once 'once.inc';
$once_second = include_once 'once.inc';
echo "once=", $once_first, ",", $once_second, ",", $count, "\n";

$required = require 'required.inc';
$required_once = require_once 'required.inc';
echo "require=", $required, ",", $required_once, ",", $required_side;
