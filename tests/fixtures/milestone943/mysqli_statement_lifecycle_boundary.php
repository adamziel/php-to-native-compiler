<?php
$stmt_init = "mysqli_stmt_init";
$prepare = "mysqli_prepare";

echo function_exists($stmt_init) ? "yes" : "no";
echo "|";
echo is_callable($stmt_init) ? "stmt-callable" : "stmt-missing";
echo "|";
echo is_callable($prepare) ? "prepare-callable" : "prepare-missing";

$handle = mysqli_init();
mysqli_stmt_init($handle);
