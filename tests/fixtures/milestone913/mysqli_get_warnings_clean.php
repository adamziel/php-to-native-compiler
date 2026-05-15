<?php
$warnings = "mysqli_get_warnings";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($dbh, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");

echo function_exists($warnings) ? "yes" : "no";
echo "\n";
echo is_callable($warnings) ? "callable" : "missing";
echo "\n";
echo mysqli_get_warnings($dbh) === false ? "false" : "warning";
echo "\n";
echo $warnings($dbh) === false ? "false" : "warning";
