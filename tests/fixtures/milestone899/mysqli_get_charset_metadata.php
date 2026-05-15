<?php
$call = "mysqli_get_charset";
$dbh = mysqli_init();
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_set_charset($dbh, "utf8mb4");

$charset = mysqli_get_charset($dbh);
$dynamic = $call($dbh);

echo function_exists($call) ? "yes" : "no";
echo "\n";
echo is_callable($call) ? "callable" : "missing";
echo "\n";
echo $charset->charset;
echo "\n";
echo $charset->collation;
echo "\n";
echo $charset->dir === "" ? "builtin" : "external";
echo "\n";
echo $charset->min_length;
echo "\n";
echo $charset->max_length;
echo "\n";
echo $charset->number;
echo "\n";
echo $charset->state;
echo "\n";
echo $dynamic->charset;
