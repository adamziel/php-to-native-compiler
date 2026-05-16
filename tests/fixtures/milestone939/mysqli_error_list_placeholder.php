<?php
$errors = "mysqli_error_list";
$dbh = mysqli_init();
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($errors) ? "yes" : "no";
echo "\n";
echo is_callable($errors) ? "callable" : "missing";
echo "\n";
echo count(mysqli_error_list($dbh));
echo "\n";
echo count($errors($dbh));
echo "\n";
echo mysqli_errno($dbh);
echo "\n";
echo mysqli_error($dbh);
