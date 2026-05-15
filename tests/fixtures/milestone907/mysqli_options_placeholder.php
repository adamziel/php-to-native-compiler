<?php
$call = "mysqli_options";
$dbh = mysqli_init();

echo function_exists($call) ? "yes" : "no";
echo "\n";
echo is_callable($call) ? "callable" : "missing";
echo "\n";
echo MYSQLI_OPT_INT_AND_FLOAT_NATIVE;
echo "\n";
echo mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true) ? "set" : "failed";
echo "\n";
echo $call($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, 1) ? "set" : "failed";
