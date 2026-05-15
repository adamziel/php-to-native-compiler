<?php
$commit = "mysqli_commit";
$rollback = "mysqli_rollback";
echo function_exists($commit) ? "yes" : "no";
echo "|";
echo is_callable($rollback) ? "callable" : "missing";
echo "\n";

$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

mysqli_begin_transaction($handle);
echo mysqli_commit($handle) ? "commit" : "failed";
echo "\n";

mysqli_begin_transaction($handle, 0, "wp");
echo mysqli_rollback($handle, 0, "wp") ? "rollback" : "failed";
echo "\n";

echo $commit($handle, 0, null) ? "dynamic-commit" : "failed";
echo "\n";
echo $rollback($handle, 0, null) ? "dynamic-rollback" : "failed";
