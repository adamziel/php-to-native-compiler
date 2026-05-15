<?php
$errno = "mysqli_connect_errno";
$error = "mysqli_connect_error";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($errno) ? "yes" : "no";
echo "\n";
echo is_callable($error) ? "callable" : "missing";
echo "\n";
echo mysqli_connect_errno();
echo "\n";
echo mysqli_connect_error() === null ? "null" : mysqli_connect_error();
echo "\n";
echo $errno();
echo "\n";
echo $error() === null ? "null" : $error();
