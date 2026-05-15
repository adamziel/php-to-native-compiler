<?php
$store = "mysqli_store_result";
$use = "mysqli_use_result";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($dbh, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");

echo function_exists($store) ? "yes" : "no";
echo "\n";
echo is_callable($use) ? "callable" : "missing";
echo "\n";
echo mysqli_store_result($dbh) === false ? "no-store" : "stored";
echo "\n";
echo mysqli_use_result($dbh) === false ? "no-use" : "using";
echo "\n";
echo $store($dbh) === false ? "dynamic-store-clean" : "dynamic-store";
echo "\n";
echo $use($dbh) === false ? "dynamic-use-clean" : "dynamic-use";
