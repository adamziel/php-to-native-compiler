<?php
$handle = mysqli_init();
$stmt = mysqli_stmt_init($handle);
echo mysqli_stmt_errno($stmt);
echo "|";
echo mysqli_stmt_error($stmt) === "" ? "empty" : "non-empty";
echo "|";
echo mysqli_stmt_sqlstate($stmt);
echo "|";
echo mysqli_stmt_warning_count($stmt);
echo "|";
echo mysqli_stmt_get_warnings($stmt) === false ? "false" : "warning";
echo "|";
echo count(mysqli_stmt_error_list($stmt));
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_stmt_insert_id($stmt);
