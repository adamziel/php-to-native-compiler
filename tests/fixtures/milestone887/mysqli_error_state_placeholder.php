<?php
$sqlstate = "mysqli_sqlstate";
$warnings = "mysqli_warning_count";
echo function_exists($sqlstate) ? "yes" : "no";
echo "|";
echo is_callable($warnings) ? "callable" : "missing";
echo "\n";

$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

echo mysqli_errno($handle);
echo "\n";
echo mysqli_error($handle);
echo "\n";
echo mysqli_sqlstate($handle);
echo "\n";
echo mysqli_warning_count($handle);
echo "\n";
echo $sqlstate($handle);
echo "\n";
echo $warnings($handle);
