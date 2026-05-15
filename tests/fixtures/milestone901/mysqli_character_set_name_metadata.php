<?php
$call = "mysqli_character_set_name";
$dbh = mysqli_init();
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_set_charset($dbh, "utf8mb4");

echo function_exists($call) ? "yes" : "no";
echo "\n";
echo is_callable($call) ? "callable" : "missing";
echo "\n";
echo mysqli_character_set_name($dbh);
echo "\n";
echo $call($dbh);
