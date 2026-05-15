<?php
$info = "mysqli_info";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($dbh, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");

echo function_exists($info) ? "yes" : "no";
echo "\n";
echo is_callable($info) ? "callable" : "missing";
echo "\n";
echo mysqli_info($dbh) === null ? "null" : mysqli_info($dbh);
echo "\n";
echo $info($dbh) === null ? "null" : $info($dbh);
